"""Plot checked numeric CSV evidence; optional matplotlib, never a Rust dependency."""
import argparse
import csv
from pathlib import Path


def records(path):
    with path.open(newline='') as stream:
        return list(csv.DictReader(stream))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence_directory', type=Path)
    args = parser.parse_args()
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    import numpy as np
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10, 'svg.fonttype': 'none',
                         'svg.hashsalt': 'airust-public-resource-evidence-v1',
                         'axes.spines.top': False, 'axes.spines.right': False,
                         'axes.axisbelow': True, 'figure.facecolor': 'white'})
    root = args.evidence_directory
    data = records(root / 'full-mcp-workers.csv')
    settings = ['baseline-2-16', 'candidate-2-1-memory', 'candidate-2-1-snapshot', 'candidate-8-1-snapshot']
    labels = ['0.6.0\n2 threads / batch16', 'candidate\n2 threads / batch1', 'candidate + file copy\n2 threads / batch1', 'candidate + file copy\n8 threads / batch1']
    fig, ax = plt.subplots(figsize=(10.5, 6.2))
    x = np.arange(len(settings))
    for campaign, color, offset in [(1, '#235d91', -.19), (2, '#8ab6d6', .19)]:
        values = [float(next(v for v in data if v['setting'] == setting and int(v['campaign']) == campaign)['p95_ms']) for setting in settings]
        bars = ax.bar(x + offset, values, .36, label=f'Campaign {campaign}', color=color)
        ax.bar_label(bars, fmt='%.1f', padding=4, fontsize=9)
    ax.set_xticks(x, labels)
    ax.set_ylim(0, 1100)
    ax.set_ylabel('Warm complete MCP request p95 (ms)')
    ax.set_title('CPU reranking: unchanged model, 20 candidates, pair limit 512 tokens', pad=16)
    ax.grid(axis='y', color='#e0e5ea')
    ax.legend(frameon=False)
    fig.text(.04, .025, '150 assistant-authored reference queries; 2 timed repetitions per query/campaign. M5 Max, 64 GiB.\nRequest includes BGE-M3 + native scoring + decoding; startup and answer generation are separate.\nLocal observations only. All samples, three slower query means and numerical differences remain in the CSV files.', fontsize=8, color='#444444')
    fig.subplots_adjust(bottom=.25, left=.10, right=.98, top=.88)
    for extension in ['svg', 'png']:
        fig.savefig(root / ('rerank-complete-request.' + extension), dpi=180, metadata={'Date': None} if extension == 'svg' else None)
    plt.close(fig)

    core = records(root / 'training-snapshot-samples.csv')
    errors = records(root / 'retained-error-samples.csv')
    panels = [('24 read-only training snapshots',
               max(int(v['peak_rss_bytes']) for v in core if v['workload']=='readonly' and v['option']=='baseline'),
               max(int(v['peak_rss_bytes']) for v in core if v['workload']=='readonly' and v['option']=='candidate')),
              ('8 independently edited snapshots',
               max(int(v['peak_rss_bytes']) for v in core if v['workload']=='all_edits' and v['option']=='baseline'),
               max(int(v['peak_rss_bytes']) for v in core if v['workload']=='all_edits' and v['option']=='candidate')),
              ('128 retained JSON input errors',
               max(int(v['peak_rss_bytes']) for v in errors if v['mode']=='mcp-json' and v['name']=='published'),
               max(int(v['peak_rss_bytes']) for v in errors if v['mode']=='mcp-json' and v['name']=='fixed'))]
    fig, axes = plt.subplots(1, 3, figsize=(11.5, 5.7), sharey=True)
    for ax, (title, before, after) in zip(axes, panels):
        bars = ax.bar(['0.6.0', 'candidate'], [before/1024**2, after/1024**2], color=['#708090','#235d91'], width=.6)
        ax.bar_label(bars, fmt='%.1f', padding=5)
        ax.set_title(title, fontsize=10, pad=12)
        ax.grid(axis='y', color='#e0e5ea')
        ax.set_ylim(0, 500)
    axes[0].set_ylabel('Worst observed worker peak RSS (MiB)')
    fig.suptitle('Memory controls preserve original values, error categories and positions', y=.95, fontsize=13)
    fig.text(.04, .035, 'Training controls: 6000 examples, 10 workers per setting/scenario. First writes to shared data copy the collection.\nError controls: 128 rejected 2 MiB JSON strings, 3 workers per build. These are generated resource probes.\nActive native models and external services are outside these model-free measurements; no 1-GiB-device certification.', fontsize=8, color='#444444')
    fig.subplots_adjust(bottom=.24, left=.08, right=.98, top=.83, wspace=.12)
    for extension in ['svg', 'png']:
        fig.savefig(root / ('memory-controls.' + extension), dpi=180, metadata={'Date': None} if extension == 'svg' else None)
    plt.close(fig)

    life = records(root / 'native-lifecycle-rss.csv')
    fig, ax = plt.subplots(figsize=(10, 5.8))
    for name, label, color in [('native-lifecycle', 'Default allocator: original failed control', '#b45f4d'),
                               ('native-lifecycle-physical-followup', 'Default allocator: failed follow-up', '#cc9d7a'),
                               ('native-lifecycle-space-efficient', 'Process launch: MallocSpaceEfficient=1', '#235d91')]:
        points = sorted([v for v in life if v['control']==name and v['stage']=='dropped'], key=lambda v:int(v['cycle']))
        ax.plot([int(v['cycle'])+1 for v in points], [int(v['rss_bytes'])/1024**2 for v in points], marker='o', label=label, color=color)
    ax.set_ylim(bottom=0)
    ax.set_xlabel('Native model load / cancel / recovery / drop cycle')
    ax.set_ylabel('RSS after drop (MiB)')
    ax.set_title('Native lifecycle: allocator retention is a separate resource boundary', pad=16)
    ax.grid(color='#e0e5ea')
    ax.legend(frameon=False, loc='lower right', fontsize=9)
    fig.text(.05, .025, 'Same native binary, eight cycles each. Default controls failed the frozen 64 MiB post-warm RSS growth budget.\nThe opt-in startup policy passed (7.6 MB range); vmmap reported 26.5M physical footprint after drop.\nRSS includes reusable pages. No global environment change, active-model 1 GiB claim or proof of absence of leaks.', fontsize=8, color='#444444')
    fig.subplots_adjust(bottom=.24, left=.11, right=.98, top=.88)
    for extension in ['svg', 'png']:
        fig.savefig(root / ('native-lifecycle.' + extension), dpi=180, metadata={'Date': None} if extension == 'svg' else None)
    plt.close(fig)


if __name__ == '__main__':
    main()
