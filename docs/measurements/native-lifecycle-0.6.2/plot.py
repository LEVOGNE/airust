"""Plot the exported numeric evidence; no private text or new measurements."""
import csv
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

OUT = Path(__file__).resolve().parent
with (OUT/'events.csv').open() as stream:
    events = list(csv.DictReader(stream))
with (OUT/'comparisons.csv').open() as stream:
    comparisons = {r['name']: r for r in csv.DictReader(stream)}
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'airust-native-lifecycle-v1'})
fig, axes = plt.subplots(1, 2, figsize=(12.8, 4.7), layout='constrained')
colors = {'baseline':'#ab3f37','candidate':'#2166ac'}
for suffix, style, alpha in [('serial', '--', .6), ('alternating', '-', 1)]:
    names = [('final-baseline-memory', 'baseline'), ('final-isolated-snapshot', 'candidate')] if suffix=='serial' else [('alternating-baseline','baseline'),('alternating-candidate','candidate')]
    for name, role in names:
        rows = [r for r in events if r['name']==name and r['stage']=='dropped']
        axes[0].plot([int(r['cycle'])+1 for r in rows], [int(r['parent_rss_bytes'])/1048576 for r in rows],
                     linestyle=style, color=colors[role], alpha=alpha, label=f'{role}, {suffix}')
axes[0].set(yscale='log', xlabel='Load/use/cancel/recover/drop cycle', ylabel='Parent RSS after drop (MiB, log scale)', title='Unloaded parent: process lifetime ends retention')
axes[0].set_ylim(2.8, 3200)
axes[0].grid(alpha=.2, which='both')
axes[0].legend(loc='center right', fontsize=8)
labels = ['Primary\nserial', 'Confirmation\nserial', 'Alternating\ndiagnostic']
keys = ['primary-selected-vs-default','serial-confirmation','alternating-diagnostic']
x = list(range(3))
for offset, metric, color, label in [(-.18,'load_ms','#8a6d3b','Load p95 ratio'),(.18,'recovery_ms','#2166ac','Warm recovery p95 ratio')]:
    values = [float(comparisons[k][metric+'_p95_ratio']) for k in keys]
    bars = axes[1].bar([v+offset for v in x],values,width=.34,color=color,label=label)
    text_labels = [f'{v:.3f}' + ('\nFAILED' if metric == 'recovery_ms' and i == 1 else '') for i,v in enumerate(values)]
    annotations = axes[1].bar_label(bars,labels=text_labels,padding=3,fontsize=8)
    if metric == 'recovery_ms':
        annotations[1].set_color('#ab3f37')
axes[1].axhline(1, color='#666666', linewidth=1)
axes[1].axhline(1.1, color='#ab3f37', linewidth=1, linestyle='--', label='Warm cost gate (1.10)')
axes[1].set(xticks=x, xticklabels=labels, ylabel='Candidate / baseline p95', ylim=(0,1.66), title='Keep startup costs and the failed latency control')
axes[1].legend(loc='lower left', fontsize=8)
fig.suptitle('AIRust 0.6.2 development — matched MiniLM lifecycle controls',fontsize=13)
fig.text(.01,-.055,'Same-host measurements: M5 Max / 64 GiB, 2 CPU threads, 20 candidates, outer batch 1. Four synthetic cases repeated; no independent relevance labels.\nActive native model memory still exceeds 1 GiB. Alternating control holds two parent processes; it does not erase the failed serial confirmation.',fontsize=8,color='#444444')
fig.savefig(OUT/'lifecycle.png',dpi=160,bbox_inches='tight',metadata={'Software':'Matplotlib; frozen numeric CSV'})
fig.savefig(OUT/'lifecycle.svg',bbox_inches='tight',metadata={'Date':None,'Creator':'AIRust lifecycle numeric evidence'})
