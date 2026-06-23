import { useEffect, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { generateReport, listReports } from '../api/tauri';
import { TEMPLATES } from '../lib/constants';
import type { Report } from '../lib/types';

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
    <div className="reports-page">
      <div className="reports-left">
        <div className="reports-form">
          <label>
            Period
            <select
              value={periodType}
              onChange={(e) => setPeriodType(e.target.value as 'daily' | 'weekly')}
            >
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
            </select>
          </label>
          <label>
            Date
            <input
              type="date"
              value={periodStart}
              onChange={(e) => setPeriodStart(e.target.value)}
            />
          </label>
          <label>
            Template
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
            {generateMutation.isPending ? 'Generating…' : 'Generate Report'}
          </button>
          {generateMutation.isError && (
            <p className="reports-error">Failed to generate report.</p>
          )}
        </div>

        <div className="reports-history">
          <h3>History</h3>
          {reportsQuery.isLoading && <p className="reports-empty">Loading…</p>}
          {!reportsQuery.isLoading && reports.length === 0 && (
            <p className="reports-empty">No reports yet.</p>
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
                  {copied ? 'Copied!' : 'Copy'}
                </button>
                <button className="btn-sm" onClick={handleExport}>
                  Export .md
                </button>
              </div>
            </div>
            <pre className="reports-preview-content">{selectedReport.content_markdown}</pre>
          </>
        ) : (
          <div className="reports-empty-preview">
            <p>Select a report or generate a new one.</p>
          </div>
        )}
      </div>
    </div>
  );
}
