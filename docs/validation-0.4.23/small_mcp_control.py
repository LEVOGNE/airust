"""Preplanned small stdio MCP comparison; all versions, cases and trials retained."""
import hashlib
import json
import pathlib
import subprocess
import sys
import benchmark as common
from host_monitor import HostMonitor

ROOT,BASE,WORKERS=common.ROOT,common.BASE,common.WORKERS
read,save,stats=common.read,common.save,common.stats

def main(campaign):
    assert campaign in ['standard','confirmation']
    common.verify()
    versions={'reference':'0.4.21','baseline':'0.4.22','candidate':'0.4.23'}
    arms=list(versions)
    binaries={arm:WORKERS/('mcp-pre-regression-0421' if arm=='reference' else 'mcp-'+arm)for arm in arms}
    cases=[(label+('' if progress else '-no-progress'),label,progress)for progress in [True,False]for label in ['tiny','tiny4','tiny16']]
    queries=BASE/'mcp-queries.json'
    files=[queries,*[BASE/(name+'-mcp.json')for name in ['tiny','tiny4','tiny16']]]
    before={str(p):hashlib.sha256(p.read_bytes()).hexdigest()for p in files}
    prefix='small-mcp-'+campaign
    reports=[];compared=0
    with HostMonitor(BASE/(prefix+'-host-samples.json')):
        for case,label,progress in cases:
            for triple in range(7):
                order=arms[triple%3:]+arms[:triple%3]
                if triple%2:order.reverse()
                transcripts={}
                for arm in order:
                    folder=BASE/prefix/f'{case}-{triple}-{arm}';folder.mkdir(parents=True)
                    args=[sys.executable,'-B',str(pathlib.Path(__file__).with_name('mcp_probe.py')),'--binary',str(binaries[arm]),'--knowledge',str(BASE/(label+'-mcp.json')),'--queries',str(queries),'--report',str(folder/'report.json'),'--transcript',str(folder/'transcript-private.json')]
                    if not progress:args.append('--no-progress')
                    with(folder/'probe.log').open('w')as log:subprocess.run(args,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=240)
                    validation=subprocess.run([str(WORKERS/'schema'),str(folder/'transcript-private.json'),str(folder/'schema.json')],capture_output=True,text=True,timeout=120)
                    (folder/'schema.log').write_text(validation.stdout+validation.stderr);assert validation.returncode==0
                    report=json.loads((folder/'report.json').read_text());assert report['version']==versions[arm]
                    reports.append({'case':case,'arm':arm,'triple':triple,**report,'schema_validation':json.loads((folder/'schema.json').read_text())})
                    transcripts[arm]=json.loads((folder/'transcript-private.json').read_text())
                for arm in arms:
                    for tool in transcripts[arm]['tools']:
                        assert tool['outputSchema']['$id']==f"urn:airust:mcp:{versions[arm]}:{tool['name']}"
                        tool['outputSchema']['$id']=f"urn:airust:mcp:VERSION:{tool['name']}"
                assert all(transcripts[a]['tools']==transcripts['candidate']['tools']for a in arms)
                values={a:{row['request_id']:row['result']['structuredContent']for row in d['results']}for a,d in transcripts.items()}
                for a in arms:
                    for value in values[a].values():
                        if value.get('operation')=='airust_status':assert value['data'].pop('version')==versions[a]
                for a in arms[:-1]:assert values[a]==values['candidate'];compared+=len(values[a])
            print(json.dumps({'stage':prefix,'case':case,'complete_response_pairs':compared}),flush=True)
    summary={}
    for case,_,_ in cases:
        summary[case]={}
        for arm in arms:
            rows=[x for x in reports if x['case']==case and x['arm']==arm]
            output={'version':versions[arm],'startup_ms':stats([x['startup_to_initialized_ms']for x in rows]),'peak_rss_bytes':stats([x['server_peak_rss_bytes']for x in rows])}
            for phase,field in [('initial','initial_search_samples'),('repeated','search_samples')]:
                output[phase+'_ms']={alg:stats([x['milliseconds']for row in rows for x in row[field]if x['algorithm']==alg])for alg in ['bm25','tfidf']}
                output[phase+'_per_query_ms']={str(q):{alg:stats([x['milliseconds']for row in rows for x in row[field]if x['algorithm']==alg and x['query_number']==q])for alg in ['bm25','tfidf']}for q in range(10)}
                output[phase+'_process_sum_ms']={alg:stats([sum(x['milliseconds']for x in row[field]if x['algorithm']==alg)for row in rows])for alg in ['bm25','tfidf']}
            summary[case][arm]=output
    validated=sum(row['schema_validation']['actual_results_validated']for row in reports)
    assert len(reports)==126 and compared==5712 and validated==8568
    assert all(hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()==h for p,h in before.items())
    proof={'campaign':campaign,'versions':versions,'cases':len(cases),'triples_per_case':7,'processes':len(reports),'actual_schema_results_validated':validated,'complete_response_pairs':compared,'all_complete_responses_equal':True,'inputs_unchanged':True,'scope':'Preplanned independent small MCP campaign,1/4/16entries each with/withoutprogress; fresh exactthree-versionbinaries. Allinitial/repeated/query/ranker/RSS/startupsamplesretained;notpooledwithordinaryorothercampaign.'}
    for suffix,value in [('reports',reports),('summary',summary),('compatibility',proof)]:save(prefix+'-'+suffix+'.json',value)
    print(json.dumps(proof),flush=True)

if __name__=='__main__':main(sys.argv[1])
