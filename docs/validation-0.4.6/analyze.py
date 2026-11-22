import collections, hashlib, json, math, pathlib, re, sys, unicodedata
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from benchmark import BASE,read,save,np,verify
def stats(values):
    if not values:return None
    a=np.asarray(values,dtype=float)
    return {'n':len(values),'mean':float(a.mean()),'median':float(np.median(a)),'p95':float(np.percentile(a,95)),'min':float(a.min()),'max':float(a.max())}
def bootstrap(a,c):
    d=np.asarray(c,dtype=float)-np.asarray(a,dtype=float);rng=np.random.default_rng(4606)
    means=d[rng.integers(0,len(d),size=(10000,len(d)))].mean(axis=1)
    return {'candidate_minus_baseline':float(d.mean()),'paired_percentile_95_ci':[float(np.percentile(means,2.5)),float(np.percentile(means,97.5))],'resamples':10000}
def main():
    verify();questions=read('questions.json');qmap={q['id']:q for q in questions};rows=[]
    for p in sorted((BASE/'answers').glob('*.json')):
        r=json.loads(p.read_text());q=qmap[r['query_id']];raw=r['response']['message']['content'];data=None
        try:data=json.loads(raw)
        except (ValueError,TypeError):pass
        shape=isinstance(data,dict) and set(data)=={'answer','citations'} and (data['answer'] is None or isinstance(data['answer'],str)) and isinstance(data['citations'],list) and all(isinstance(c,dict) and set(c)=={'id','quote'} and all(isinstance(c[k],str) for k in ['id','quote']) for c in data['citations'])
        answer=data['answer'] if shape else None;citations=data['citations'] if shape else []
        delivered={p['id'] for p in r['passages']};ids={c['id'] for c in citations}
        inline=set(re.findall(r'\bd\d{3,}\b',answer)) if isinstance(answer,str) else set()
        inline_valid=inline<=ids and inline<=delivered
        def fact(pattern):
            if re.fullmatch(r'\w+',pattern):pattern=r'(?<!\w)'+pattern+r'(?!\w)'
            return bool(re.search(pattern,unicodedata.normalize('NFKC',answer),flags=re.I))
        facts=shape and isinstance(answer,str) and bool(answer.strip()) and all(fact(pattern) for pattern in q['checks'])
        check=r['citation_validation']['validation'];report=check.get('report',{})
        proper_abstain=bool(check.get('admitted') and report.get('abstained'))
        quotes=bool(check.get('admitted') and report.get('quotes_verified'))
        gold=set(q['gold_ids']);ranked=[p['id'] for p in r['passages']]
        gold_in_context=any(p['id'] in gold and q.get('gold_quote','') in p['text'] for p in r['passages']) if q['answerable'] else None
        gold_cited=any(p['id'] in gold & ids and q.get('gold_quote','') in p['text'] for p in r['passages']) if q['answerable'] else None
        costs=r['response']
        row={'query_id':q['id'],'partition':q['partition'],'project':q['project'],'arm':r['arm'],'position':r['position'],'answerable':q['answerable'],'schema_shape':shape,'answer_admitted':bool(check.get('admitted')),'references_valid':bool(report.get('references_valid')),'quotes_verified':quotes,'proper_abstention':proper_abstain,'fact_match':bool(facts) if q['answerable'] else None,'gold_in_context':gold_in_context,'gold_cited':gold_cited,'inline_consistent':inline_valid,'inline_present':bool(inline),'fact_and_quote_proxy':bool(facts and quotes and inline_valid) if q['answerable'] else proper_abstain,'reference_answer_proxy':bool(facts and quotes and gold_cited and inline_valid) if q['answerable'] else proper_abstain,'llm_wall_s':r['llm_wall_s'],'pipeline_wall_s':r['pipeline_wall_s'],'context_ms':r['context_ms'],'citation_validation_wall_s':r['citation_validation_wall_s'],'citation_check_ms':r['citation_validation']['check_ms']+r['citation_validation'].get('admission_ms',0),'embedding_wall_s':r['embedding_wall_s'],'prompt_tokens':costs.get('prompt_eval_count',0),'cached_prompt_tokens':costs.get('prompt_eval_cached_count',0),'generated_tokens':costs.get('eval_count',0),'context_utf8_bytes':sum(len(p['text'].encode()) for p in r['passages']),'done_reason':costs.get('done_reason'),'citation_statuses':[c['status'] for c in report.get('checks',[])],'quotes_over_prompt_character_limit':sum(len(c['quote'])>180 for c in citations)}
        rows.append(row)
    save('scored-rows.json',rows);summary={}
    metrics=['schema_shape','answer_admitted','references_valid','quotes_verified','proper_abstention','fact_match','gold_in_context','gold_cited','inline_consistent','inline_present','fact_and_quote_proxy','reference_answer_proxy']
    costs=['pipeline_wall_s','llm_wall_s','context_ms','citation_validation_wall_s','citation_check_ms','embedding_wall_s','prompt_tokens','cached_prompt_tokens','generated_tokens','context_utf8_bytes']
    for partition in ['all','exposed_regression','new_holdout']:
        selected=[r for r in rows if partition=='all' or r['partition']==partition];arms={}
        for arm in 'ACS':
            ar=[r for r in selected if r['arm']==arm];positive=[r for r in ar if r['answerable']];negative=[r for r in ar if not r['answerable']]
            arms[arm]={'completed':len(ar),'positive_n':len(positive),'negative_n':len(negative),'counts':{m:sum(bool(r[m]) for r in ar) for m in metrics},'negative_correct_abstentions':sum(r['proper_abstention'] for r in negative),'positive_unnecessary_abstentions':sum(r['proper_abstention'] for r in positive),'length_stops':sum(r['done_reason']=='length' for r in ar),'quote_length_violations':sum(r['quotes_over_prompt_character_limit'] for r in ar),'citation_statuses':dict(collections.Counter(c for r in ar for c in r['citation_statuses'])),'costs':{k:stats([r[k] for r in ar]) for k in costs},'token_totals':{k:sum(r[k] for r in ar) for k in ['prompt_tokens','cached_prompt_tokens','generated_tokens']}}
        summary[partition]={'arms':arms}
        ids=[q['id'] for q in questions if partition=='all' or q['partition']==partition];lookup={(r['query_id'],r['arm']):r for r in selected}
        if all((i,a) in lookup for i in ids for a in 'ACS'):
            summary[partition]['paired_A_C']={k:bootstrap([lookup[i,'A'][k] for i in ids],[lookup[i,'C'][k] for i in ids]) for k in ['fact_and_quote_proxy','reference_answer_proxy','prompt_tokens','pipeline_wall_s']}
            common=[i for i in ids if qmap[i]['answerable'] and lookup[i,'A']['fact_and_quote_proxy'] and lookup[i,'C']['fact_and_quote_proxy']]
            summary[partition]['both_positive_proxy_pass']={'n':len(common),'costs':{a:{k:stats([lookup[i,a][k] for i in common]) for k in ['pipeline_wall_s','prompt_tokens','generated_tokens']} for a in 'AC'}}
    save('summary.json',summary)
    print(json.dumps({'answers':len(rows),'partitions':{p:{a:{'n':d['completed'],'gold':d['counts']['gold_in_context'],'facts':d['counts']['fact_match'],'fact_quote_proxy':d['counts']['fact_and_quote_proxy'],'reference_proxy':d['counts']['reference_answer_proxy'],'negative_abstentions':d['negative_correct_abstentions']} for a,d in s['arms'].items()} for p,s in summary.items()}}))
if __name__=='__main__':main()
