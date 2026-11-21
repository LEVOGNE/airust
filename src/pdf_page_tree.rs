use pdf_oxide::object::{Object, ObjectRef};
use pdf_oxide::PdfDocument;
use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Limits {
    depth: usize,
    references: usize,
    pages: Option<usize>,
}

enum Step {
    Visit(ObjectRef, usize),
    Continue(Branch),
}

struct Branch {
    reference: ObjectRef,
    declared: Option<i64>,
    start: usize,
    depth: usize,
    children: std::vec::IntoIter<ObjectRef>,
}

pub(super) fn validated_page_count(
    doc: &PdfDocument,
    maximum_pages: Option<usize>,
) -> Result<usize, String> {
    let catalog = doc.catalog().map_err(|error| error.to_string())?;
    let root = catalog
        .as_dict()
        .and_then(|dict| dict.get("Pages"))
        .and_then(Object::as_reference)
        .ok_or_else(|| "catalog /Pages must be an indirect reference".to_owned())?;
    count_tree(
        root,
        Limits {
            depth: 50,
            references: 1_000_000,
            pages: maximum_pages,
        },
        |reference| {
            doc.load_object(reference)
                .map_err(|error| error.to_string())
        },
    )
}

fn take_references(work: &mut usize, count: usize, maximum: usize) -> Result<(), String> {
    *work = work
        .checked_add(count)
        .filter(|&value| value <= maximum)
        .ok_or_else(|| format!("page tree reference limit exceeded (maximum {maximum})"))?;
    Ok(())
}

fn declared_count(
    value: Option<&Object>,
    work: &mut usize,
    limits: Limits,
    load: &mut impl FnMut(ObjectRef) -> Result<Object, String>,
) -> Result<Option<i64>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if let Object::Integer(value) = value {
        return if *value >= 0 {
            Ok(Some(*value))
        } else {
            Err("page tree /Count must be a nonnegative integer".to_owned())
        };
    }
    let mut reference = value
        .as_reference()
        .ok_or_else(|| "page tree /Count must be a nonnegative integer".to_owned())?;
    let mut visited = HashSet::new();
    for _ in 0..limits.depth {
        if !visited.insert(reference) {
            return Err("cyclic page tree /Count reference".to_owned());
        }
        take_references(work, 1, limits.references)?;
        match load(reference)? {
            Object::Integer(value) if value >= 0 => return Ok(Some(value)),
            Object::Reference(next) => reference = next,
            _ => return Err("page tree /Count must be a nonnegative integer".to_owned()),
        }
    }
    Err(format!(
        "page tree /Count reference depth limit exceeded (maximum {})",
        limits.depth
    ))
}

fn count_tree(
    root: ObjectRef,
    limits: Limits,
    mut load: impl FnMut(ObjectRef) -> Result<Object, String>,
) -> Result<usize, String> {
    let mut work = 0;
    take_references(&mut work, 1, limits.references)?;
    let mut pending = vec![Step::Visit(root, 0)];
    let mut visited = HashSet::new();
    let mut pages = 0usize;
    while let Some(step) = pending.pop() {
        match step {
            Step::Continue(mut branch) => {
                if let Some(child) = branch.children.next() {
                    let depth = branch.depth + 1;
                    pending.push(Step::Continue(branch));
                    pending.push(Step::Visit(child, depth));
                } else {
                    let actual = pages - branch.start;
                    if let Some(declared) = branch.declared {
                        if i64::try_from(actual).ok() != Some(declared) {
                            return Err(format!(
                                "page count mismatch at object {} {}: declared {declared}, actual {actual}; no partial import returned",
                                branch.reference.id, branch.reference.gen
                            ));
                        }
                    }
                }
            }
            Step::Visit(reference, depth) => {
                if depth > limits.depth {
                    return Err(format!(
                        "page tree depth limit exceeded (maximum {})",
                        limits.depth
                    ));
                }
                if !visited.insert(reference.id) {
                    return Err(format!(
                        "duplicate or cyclic page tree reference at object {} {}",
                        reference.id, reference.gen
                    ));
                }
                let node = load(reference)?;
                let dict = node.as_dict().ok_or_else(|| {
                    format!(
                        "page tree object {} {} must be a dictionary",
                        reference.id, reference.gen
                    )
                })?;
                match dict.get("Type").and_then(Object::as_name) {
                    Some("Page") if depth != 0 => {
                        pages = pages.checked_add(1).ok_or("page count overflow")?;
                        if let Some(maximum) = limits.pages {
                            if pages > maximum {
                                return Err(format!(
                                    "PDF import limit exceeded: page count ({pages}; maximum {maximum}); no partial import returned"
                                ));
                            }
                        }
                    }
                    Some("Pages") => {
                        let declared =
                            declared_count(dict.get("Count"), &mut work, limits, &mut load)?;
                        let kids = dict
                            .get("Kids")
                            .and_then(Object::as_array)
                            .ok_or_else(|| "page tree /Kids must be an array".to_owned())?;
                        take_references(&mut work, kids.len(), limits.references)?;
                        let children = kids
                            .iter()
                            .map(|kid| {
                                kid.as_reference().ok_or_else(|| {
                                    "page tree /Kids entries must be indirect references".to_owned()
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        pending.push(Step::Continue(Branch {
                            reference,
                            declared,
                            start: pages,
                            depth,
                            children: children.into_iter(),
                        }));
                    }
                    _ => {
                        return Err(format!(
                            "invalid page tree /Type at object {} {}",
                            reference.id, reference.gen
                        ))
                    }
                }
            }
        }
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn reference(id: u32) -> ObjectRef {
        ObjectRef { id, gen: 0 }
    }

    fn leaf() -> Object {
        Object::Dictionary(HashMap::from([(
            "Type".into(),
            Object::Name("Page".into()),
        )]))
    }

    fn branch(count: Option<Object>, kids: Vec<Object>) -> Object {
        let mut dict = HashMap::from([
            ("Type".into(), Object::Name("Pages".into())),
            ("Kids".into(), Object::Array(kids)),
        ]);
        if let Some(count) = count {
            dict.insert("Count".into(), count);
        }
        Object::Dictionary(dict)
    }

    fn refs(ids: &[u32]) -> Vec<Object> {
        ids.iter()
            .map(|id| Object::Reference(reference(*id)))
            .collect()
    }

    fn limits() -> Limits {
        Limits {
            depth: 50,
            references: 1_000_000,
            pages: None,
        }
    }

    fn check(nodes: &HashMap<ObjectRef, Object>, limits: Limits) -> Result<usize, String> {
        count_tree(reference(1), limits, |key| {
            nodes
                .get(&key)
                .cloned()
                .ok_or_else(|| "missing tree object".into())
        })
    }

    fn two_pages(count: Option<Object>) -> HashMap<ObjectRef, Object> {
        HashMap::from([
            (reference(1), branch(count, refs(&[2, 3]))),
            (reference(2), leaf()),
            (reference(3), leaf()),
        ])
    }

    #[test]
    fn every_declared_count_is_compared_with_actual_leaves() {
        for declared in [0, 1, 2, 3, i64::MAX] {
            let result = check(&two_pages(Some(Object::Integer(declared))), limits());
            if declared == 2 {
                assert_eq!(result.unwrap(), 2);
            } else {
                let error = result.unwrap_err();
                assert!(error.contains("page count mismatch"));
                assert!(error.contains(&format!("declared {declared}, actual 2")));
            }
        }
        assert_eq!(check(&two_pages(None), limits()).unwrap(), 2);
    }

    #[test]
    fn root_total_cannot_hide_inconsistent_subtree_counts() {
        let nodes = HashMap::from([
            (
                reference(1),
                branch(Some(Object::Integer(3)), refs(&[2, 3])),
            ),
            (
                reference(2),
                branch(Some(Object::Integer(1)), refs(&[4, 5])),
            ),
            (reference(3), branch(Some(Object::Integer(2)), refs(&[6]))),
            (reference(4), leaf()),
            (reference(5), leaf()),
            (reference(6), leaf()),
        ]);
        let error = check(&nodes, limits()).unwrap_err();
        assert!(error.contains("object 2 0: declared 1, actual 2"));
    }

    #[test]
    fn mixed_empty_and_nonflat_branches_are_counted_by_type() {
        let nodes = HashMap::from([
            (
                reference(1),
                branch(Some(Object::Integer(2)), refs(&[2, 3])),
            ),
            (reference(2), branch(Some(Object::Integer(0)), refs(&[]))),
            (
                reference(3),
                branch(Some(Object::Integer(2)), refs(&[4, 5])),
            ),
            (reference(4), leaf()),
            (reference(5), leaf()),
        ]);
        assert_eq!(check(&nodes, limits()).unwrap(), 2);
        assert_eq!(
            check(
                &HashMap::from([(reference(1), branch(Some(Object::Integer(0)), vec![]))]),
                limits()
            )
            .unwrap(),
            0
        );
    }

    #[test]
    fn invalid_count_types_and_negative_values_fail() {
        for count in [
            Object::Integer(-1),
            Object::Real(2.0),
            Object::Boolean(true),
            Object::Null,
            Object::Name("2".into()),
        ] {
            assert!(check(&two_pages(Some(count)), limits())
                .unwrap_err()
                .contains("nonnegative integer"));
        }
    }

    #[test]
    fn indirect_counts_are_checked_and_bounded() {
        let mut nodes = two_pages(Some(Object::Reference(reference(4))));
        nodes.insert(reference(4), Object::Reference(reference(5)));
        nodes.insert(reference(5), Object::Integer(2));
        assert_eq!(check(&nodes, limits()).unwrap(), 2);
        assert!(check(
            &nodes,
            Limits {
                references: 4,
                ..limits()
            }
        )
        .is_err());
        nodes.insert(reference(5), Object::Reference(reference(4)));
        assert!(check(&nodes, limits()).unwrap_err().contains("cyclic"));
        nodes.insert(reference(5), Object::Integer(1));
        assert!(check(&nodes, limits()).unwrap_err().contains("actual 2"));
    }

    #[test]
    fn cycles_duplicates_and_generation_aliases_do_not_count_twice() {
        for kids in [
            refs(&[2, 2]),
            refs(&[1, 2]),
            vec![
                Object::Reference(reference(2)),
                Object::Reference(ObjectRef { id: 2, gen: 1 }),
            ],
        ] {
            let nodes = HashMap::from([
                (reference(1), branch(Some(Object::Integer(2)), kids)),
                (reference(2), leaf()),
            ]);
            assert!(check(&nodes, limits())
                .unwrap_err()
                .contains("duplicate or cyclic"));
        }
    }

    #[test]
    fn invalid_or_missing_tree_nodes_never_become_empty_branches() {
        let mut nodes = two_pages(Some(Object::Integer(2)));
        nodes.remove(&reference(3));
        assert!(check(&nodes, limits()).is_err());
        for bad in [
            Object::Null,
            Object::Dictionary(HashMap::new()),
            branch(Some(Object::Integer(1)), vec![Object::Integer(3)]),
        ] {
            nodes.insert(reference(3), bad);
            assert!(check(&nodes, limits()).is_err());
        }
        assert!(check(&HashMap::from([(reference(1), leaf())]), limits()).is_err());
    }

    #[test]
    fn exact_reference_depth_and_page_limits_apply_to_actual_structure() {
        let nodes = two_pages(Some(Object::Integer(2)));
        assert_eq!(
            check(
                &nodes,
                Limits {
                    references: 3,
                    depth: 1,
                    pages: Some(2)
                }
            )
            .unwrap(),
            2
        );
        assert!(check(
            &nodes,
            Limits {
                references: 2,
                ..limits()
            }
        )
        .is_err());
        assert!(check(
            &nodes,
            Limits {
                depth: 0,
                ..limits()
            }
        )
        .is_err());
        assert!(check(
            &nodes,
            Limits {
                pages: Some(1),
                ..limits()
            }
        )
        .unwrap_err()
        .contains("maximum 1"));
        let hidden = two_pages(Some(Object::Integer(1)));
        assert!(check(
            &hidden,
            Limits {
                pages: Some(1),
                ..limits()
            }
        )
        .unwrap_err()
        .contains("page count (2; maximum 1)"));
    }

    #[test]
    fn excessive_width_is_rejected_before_scheduling_children() {
        let root = branch(
            Some(Object::Integer(1000)),
            refs(&(2..1002).collect::<Vec<_>>()),
        );
        let mut loads = 0;
        let result = count_tree(
            reference(1),
            Limits {
                references: 10,
                ..limits()
            },
            |_| {
                loads += 1;
                Ok(root.clone())
            },
        );
        assert!(result.unwrap_err().contains("reference limit"));
        assert_eq!(loads, 1);
    }

    #[test]
    fn deeply_nested_input_stops_without_recursing() {
        let mut loads = 0;
        let result = count_tree(reference(1), limits(), |key| {
            loads += 1;
            Ok(branch(Some(Object::Integer(1)), refs(&[key.id + 1])))
        });
        assert!(result.unwrap_err().contains("depth limit"));
        assert_eq!(loads, 51);
    }

    #[test]
    fn varied_nested_branches_keep_independent_subtree_totals() {
        fn build(
            nodes: &mut HashMap<ObjectRef, Object>,
            next: &mut u32,
            state: &mut u64,
            depth: usize,
            root: bool,
        ) -> (ObjectRef, usize) {
            let id = reference(*next);
            *next += 1;
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            if !root && (depth == 0 || (*state).is_multiple_of(3)) {
                nodes.insert(id, leaf());
                return (id, 1);
            }
            let width = (*state % 6) as usize;
            let mut children = Vec::new();
            let mut total = 0;
            for _ in 0..width {
                let (child, count) = build(nodes, next, state, depth.saturating_sub(1), false);
                children.push(Object::Reference(child));
                total += count;
            }
            nodes.insert(id, branch(Some(Object::Integer(total as i64)), children));
            (id, total)
        }
        let mut state = 202611210315;
        for _ in 0..128 {
            let mut nodes = HashMap::new();
            let (_, expected) = build(&mut nodes, &mut 1, &mut state, 5, true);
            assert_eq!(check(&nodes, limits()).unwrap(), expected);
            let Object::Dictionary(root) = nodes.get_mut(&reference(1)).unwrap() else {
                unreachable!();
            };
            root.insert("Count".into(), Object::Integer(expected as i64 + 1));
            assert!(check(&nodes, limits())
                .unwrap_err()
                .contains("page count mismatch"));
        }
    }
}
