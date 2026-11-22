import collections, hashlib, json, math, pathlib, random, re, statistics, sys, unicodedata
from run import BASE,read,save,verify,np
def distribution(xs):
    a=np.asarray(xs,dtype=float)
    return {'n':len(xs),'mean':float(a.mean()),'median':float(np.median(a)),'p95':float(np.percentile(a,95)),'min':float(a.min()),'max':float(a.max())} if len(xs) else None
def paired_bootstrap(a,b,seed=444,repeat=10000):
    d=np.asarray(b,dtype=float)-np.asarray(a,dtype=float);rng=np.random.default_rng(seed)
    means=d[rng.integers(0,len(d),size=(repeat,len(d)))].mean(axis=1)
    return {'difference_B_minus_A':float(d.mean()),'paired_percentile_95_ci':[float(np.percentile(means,2.5)),float(np.percentile(means,97.5))],'resamples':repeat}
def main():
    verify();qs=read('questions.json');qmap={q['id']:q for q in qs};rows=[]
    retrieval={(r['query_id'],r['arm']):r for r in read('retrieval-results.json')}
    for path in sorted((BASE/'answers').glob('q*.json')):
        r=json.loads(path.read_text());q=qmap[r['query_id']];response=r['response']
        raw=response.get('message',{}).get('content','');parsed=None
        try:parsed=json.loads(raw)
        except (ValueError,TypeError):pass
        valid=isinstance(parsed,dict) and set(parsed)=={'answer','citations'} and (parsed['answer'] is None or isinstance(parsed['answer'],str)) and isinstance(parsed['citations'],list) and all(isinstance(x,str) for x in parsed['citations'])
        ans=parsed.get('answer') if valid else None;citations=parsed.get('citations',[]) if valid else []
        available={p['id'] for p in r['passages']};citation_ids_valid=valid and set(citations)<=available
        abstain=valid and ans is None and not citations
        inline_ids=set(re.findall(r'\bd\d{3,}\b',ans)) if isinstance(ans,str) else set()
        def fact(pattern):
            if re.fullmatch('[A-Za-z]',pattern):pattern=r'(?<!\w)'+pattern+r'(?!\w)'
            return re.search(pattern,unicodedata.normalize('NFKC',ans),flags=re.I)
        facts=valid and isinstance(ans,str) and bool(ans.strip()) and all(fact(pattern) for pattern in q['checks'])
        gold=set(q['gold_ids']);hits=retrieval[q['id'],r['arm']]['hits'];ids=[h['id'] for h in hits]
        gold_citation=any(p['id'] in citations and p['id'] in gold and q.get('gold_quote','') in p['text'] for p in r['passages']) if q['answerable'] else False
        delivered=any(p['id'] in gold and q.get('gold_quote','') in p['text'] for p in r['passages']) if q['answerable'] else False
        row={'query_id':q['id'],'project':q['project'],'arm':r['arm'],'position':r['position'],'answerable':q['answerable'],'valid_json':valid,'citation_ids_valid':citation_ids_valid,'fact_check':bool(facts) if q['answerable'] else None,'gold_citation':gold_citation if q['answerable'] else None,'abstention':abstain,'strict_proxy_pass':bool(facts and citation_ids_valid and gold_citation) if q['answerable'] else abstain,'delivered_gold':delivered if q['answerable'] else None,'llm_wall_s':r['wall_s'],'pipeline_wall_s':r['pipeline_wall_s'],'embedding_wall_s':r['embedding_wall_s'],'retrieval_ms':r['live_retrieval']['search_ms'],'context_utf8_bytes':r['context_utf8_bytes'],'prompt_tokens':response.get('prompt_eval_count',0),'cached_prompt_tokens':response.get('prompt_eval_cached_count',0),'generated_tokens':response.get('eval_count',0),'done_reason':response.get('done_reason'),'hit1':bool(gold & set(ids[:1])) if q['answerable'] else None,'hit3':bool(gold & set(ids[:3])) if q['answerable'] else None,'hit6':bool(gold & set(ids[:6])) if q['answerable'] else None,'mrr6':next((1/(i+1) for i,x in enumerate(ids) if x in gold),0) if q['answerable'] else None}
        rows.append(row)
        row['inline_citation_ids_valid']=inline_ids<=available
        row['inline_citations_listed']=inline_ids<=set(citations)
        row['string_null_answer']=ans=='null'
    save('scored-rows.json',rows)
    summary={}
    for arm in 'ABCD':
        ar=[r for r in rows if r['arm']==arm];positive=[r for r in ar if r['answerable']];negative=[r for r in ar if not r['answerable']]
        summary[arm]={'completed':len(ar),'positive_count':len(positive),'negative_count':len(negative),'metrics':{k:sum(bool(r[k]) for r in positive) for k in ['hit1','hit3','hit6','fact_check','gold_citation','delivered_gold']},'mrr6':statistics.mean(r['mrr6'] for r in positive) if positive else None,'correct_abstentions':sum(r['abstention'] for r in negative),'strict_proxy_pass':sum(r['strict_proxy_pass'] for r in ar),'valid_json':sum(r['valid_json'] for r in ar),'valid_citation_ids':sum(r['citation_ids_valid'] for r in ar),'length_stops':sum(r['done_reason']=='length' for r in ar),'costs':{k:distribution([r[k] for r in ar]) for k in ['llm_wall_s','pipeline_wall_s','embedding_wall_s','retrieval_ms','context_utf8_bytes','prompt_tokens','cached_prompt_tokens','generated_tokens']},'tokens_total':{k:sum(r[k] for r in ar) for k in ['prompt_tokens','cached_prompt_tokens','generated_tokens']},'per_project':{p:{'n':sum(r['project']==p for r in ar),'positive_n':sum(r['project']==p for r in positive),'hit6':sum(r['project']==p and r['hit6'] for r in positive),'strict_proxy_pass':sum(r['project']==p and r['strict_proxy_pass'] for r in ar)} for p in ['airust','flasm','vecasm']}}
    comparisons={}
    conditional={}
    if all(summary[a]['completed']==100 for a in 'ABCD'):
        by={(r['query_id'],r['arm']):r for r in rows}
        for a,b in [('A','B'),('A','C'),('B','C'),('D','C')]:
            comparisons[a+'-vs-'+b]={k:paired_bootstrap([by[q['id'],a][k] for q in qs],[by[q['id'],b][k] for q in qs]) for k in ['strict_proxy_pass','pipeline_wall_s','prompt_tokens']}
            common=[q for q in qs if q['answerable'] and by[q['id'],a]['strict_proxy_pass'] and by[q['id'],b]['strict_proxy_pass']]
            conditional[a+'-vs-'+b]={'both_reference_proxy_pass_positive_questions':len(common),'costs':{arm:{k:distribution([by[q['id'],arm][k] for q in common]) for k in ['pipeline_wall_s','prompt_tokens','generated_tokens']} for arm in [a,b]}}
    warm={a:{k:distribution([r[k] for r in values]) for k in ['search_ms','ipc_wall_ms']} for a,values in read('retrieval-timings.json').items()}
    save('summary.json',{'arms':summary,'paired_comparisons':comparisons,'conditional_both_positive_proxy_pass':conditional,'warm_retrieval_500_per_arm':warm,'label_warning':'Document-grounded authored reference set; automatic exact-fact and reference-citation proxy, NOT independently adjudicated correct-answer rate.'})
    save('citation-format-diagnostics.json',{a:{'invalid_inline_source_ids':sum(not r['inline_citation_ids_valid'] for r in rows if r['arm']==a),'inline_not_listed':sum(not r['inline_citations_listed'] for r in rows if r['arm']==a),'string_null_answers':sum(r['string_null_answer'] for r in rows if r['arm']==a)} for a in 'ABCD'})
    print(json.dumps({'completed':len(rows),'arms':{a:{'n':s['completed'],'hit6':s['metrics']['hit6'],'facts':s['metrics']['fact_check'],'strict_proxy_pass':s['strict_proxy_pass'],'abstentions':s['correct_abstentions']} for a,s in summary.items()}}))
if __name__=='__main__':main()
