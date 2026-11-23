"""Check existing broad-query rejection and recovery, outside latency campaigns."""
import json
import benchmark as common
from mcp_probe import Connection

common.verify()
rows=[]
for arm,version in [('reference','0.4.20'),('baseline','0.4.21'),('candidate','0.4.22')]:
    binary=common.WORKERS/('mcp-pre-regression-0420' if arm=='reference' else 'mcp-'+arm)
    client=Connection([str(binary),'--knowledge',str(common.BASE/'wide-mcp.json')])
    try:
        initialized,_=client.request('initialize',{'protocolVersion':'2025-11-25','clientInfo':{'name':'airust-wide-bound-check','version':'1'},'capabilities':{}})
        assert initialized['result']['serverInfo']['version']==version
        client.send({'jsonrpc':'2.0','method':'notifications/initialized'})
        rejected=0
        for width in [257,512]:
            query=' '.join('term'+str(i) for i in range(width))
            assert len(query.encode())<=4096
            for algorithm in ['bm25','tfidf']:
                response,events=client.request('tools/call',{'name':'airust_search','arguments':{'query':query,'algorithm':algorithm}},progress=True)
                result=response['result'];assert result['isError']
                assert result['structuredContent']=={'schema_version':1,'error':{'code':'operation_failed','message':'retrieval budget exceeded: assessment query terms'}}
                assert json.loads(result['content'][0]['text'])==result['structuredContent']
                assert events[0]['message']=='Starting airust_search' and events[-1]['message']=='AIRust work stopped with an error'
                assert all(a['progress']<b['progress']for a,b in zip(events,events[1:]))
                rejected+=1
        status,_=client.request('tools/call',{'name':'airust_status','arguments':{}})
        assert not status['result'].get('isError',False) and status['result']['structuredContent']['data']['version']==version
        rows.append({'version':version,'rejected_queries':rejected,'no_partial_results':True,'progress_stops_with_error':True,'subsequent_status_succeeds':True})
    finally:client.close()
common.save('wide-boundary-checks.json',{'processes':3,'total_rejected_queries':12,'unchanged_256_unique_assessment_term_bound':True,'all_versions_agree':True,'rows':rows})
print('PASS existing broad-query bounds and recovery')
