import { useEffect, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { generateReport, listReports } from '../api/tauri';
import { TEMPLATES } from '../lib/constants';
import type { Report } from '../lib/types';
import { LoadingState, ErrorState, EmptyState } from '../components/StateViews';

function todayISO() {
  return new Date().toISOString().slice(0, 10);
}

function weekStartISO() {
  const d = new Date();
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  return d.toISOString().slice(0, 10);
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function exportMarkdown(report: Report) {
  const blob = new Blob([report.content_markdown], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${report.title.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.md`;
  a.click();
  URL.revokeObjectURL(url);
}

export default function Reports() {
  const queryClient = useQueryClient();
  const [periodType, setPeriodType] = useState<'daily' | 'weekly'>('daily');
  const [periodStart, setPeriodStart] = useState(todayISO());
  const [templateId, setTemplateId] = useState(TEMPLATES[0].id);
  const [selectedReportId, setSelectedReportId] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const reportsQuery = useQuery({
    queryKey: ['reports'],
    queryFn: () => listReports(),
  });

  const generateMutation = useMutation({
    mutationFn: () => generateReport(periodType, periodStart, templateId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['reports'] });
    },
  });

  useEffect(() => {
    if (periodType === 'weekly') {
      setPeriodStart(weekStartISO());
    } else {
      setPeriodStart(todayISO());
    }
  }, [periodType]);

  const reports = reportsQuery.data ?? [];
  const selectedReport = reports.find((r) => r.id === selectedReportId) ?? reports[0] ?? null;

  function handleCopy() {
    if (!selectedReport) return;
    copyToClipboard(selectedReport.content_markdown);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  function handleExport() {
    if (!selectedReport) return;
    exportMarkdown(selectedReport);
  }

  return (
    <div className="reports-wrapper">
      <header className="page-header">
        <p className="page-kicker">reports</p>
        <h2>报告</h2>
        <p>基于每日活动数据，一键生成日报或周报。</p>
      </header>
      <div className="reports-page">
        <div className="reports-left">
          <div className="reports-form">
            <label>
              周期
              <select
                value={periodType}
                onChange={(e) => setPeriodType(e.target.value as 'daily' | 'weekly')}
              >
                <option value="daily">日报</option>
                <option value="weekly">周报</option>
              </select>
            </label>
            <label>
              日期
              <input
                type="date"
                value={periodStart}
                onChange={(e) => setPeriodStart(e.target.value)}
              />
            </label>
            <label>
              模板
              <select value={templateId} onChange={(e) => setTemplateId(e.target.value)}>
                {TEMPLATES.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.label}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="btn-sm btn-primary btn-generate"
              disabled={generateMutation.isPending}
              onClick={() => generateMutation.mutate()}
            >
              {generateMutation.isPending ? '生成中…' : '生成报告'}
            </button>
            {generateMutation.isError && (
              <p className="reports-error">报告生成失败。</p>
            )}
          </div>

          <div className="reports-history">
            <h3>历史记录</h3>
            {reportsQuery.isLoading && <LoadingState message="加载报告列表..." />}
            {reportsQuery.isError && <ErrorState message="加载失败" onRetry={() => reportsQuery.refetch()} />}
            {!reportsQuery.isLoading && !reportsQuery.isError && reports.length === 0 && (
              <EmptyState message="暂无报告" />
            )}
            <ul className="reports-history-list">
              {reports.map((r) => (
                <li
                  key={r.id}
                  className={`reports-history-item ${selectedReport?.id === r.id ? 'selected' : ''}`}
                  onClick={() => setSelectedReportId(r.id)}
                >
                  <span className="reports-history-title">{r.title}</span>
                  <span className="reports-history-meta">
                    {r.period_type} · {r.period_start}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </div>

        <div className="reports-right">
          {selectedReport ? (
            <>
              <div className="reports-preview-header">
                <h3>{selectedReport.title}</h3>
                <div className="reports-preview-actions">
                  <button className="btn-sm" onClick={handleCopy}>
                    {copied ? '已复制！' : '复制'}
                  </button>
                  <button className="btn-sm" onClick={handleExport}>
                    导出 .md
                  </button>
                </div>
              </div>
              <pre className="reports-preview-content">{selectedReport.content_markdown}</pre>
            </>
          ) : (
            <div className="reports-empty-preview">
              <p>选择一份报告或生成新报告。</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
