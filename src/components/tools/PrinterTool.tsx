import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Printer, Trash2, RefreshCw, FileText } from 'lucide-react';

interface PrinterInfo {
  name: string;
  status: string;
  is_default: boolean;
  port: string;
  driver: string;
}

interface PrintJob {
  id: number;
  document: string;
  status: string;
  size: string;
}

export default function PrinterTool() {
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [selectedPrinter, setSelectedPrinter] = useState('');
  const [jobs, setJobs] = useState<PrintJob[]>([]);
  const [loading, setLoading] = useState(false);
  const [clearing, setClearing] = useState(false);

  const loadPrinters = async () => {
    setLoading(true);
    try {
      const data = await invoke<PrinterInfo[]>('get_printers');
      setPrinters(data);
      if (data.length > 0 && !selectedPrinter) {
        const def = data.find((p) => p.is_default);
        setSelectedPrinter(def?.name || data[0].name);
      }
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const loadJobs = async (printerName: string) => {
    if (!printerName) return;
    try {
      const data = await invoke<PrintJob[]>('get_print_jobs', { printer_name: printerName });
      setJobs(data);
    } catch (e) {
      console.error(e);
      setJobs([]);
    }
  };

  const handleClearQueue = async () => {
    if (!selectedPrinter) return;
    setClearing(true);
    try {
      await invoke('clear_print_queue', { printer_name: selectedPrinter });
      loadJobs(selectedPrinter);
    } catch (e) {
      console.error(e);
    }
    setClearing(false);
  };

  useEffect(() => {
    loadPrinters();
  }, []);

  useEffect(() => {
    loadJobs(selectedPrinter);
  }, [selectedPrinter]);

  const currentPrinter = printers.find((p) => p.name === selectedPrinter);

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-lg text-text-primary">打印机管理</h3>
        <button
          onClick={loadPrinters}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      {/* 打印机列表 */}
      <div className="space-y-2">
        {printers.map((printer) => (
          <button
            key={printer.name}
            onClick={() => setSelectedPrinter(printer.name)}
            className={`w-full flex items-center gap-3 p-4 rounded-xl border transition-all text-left ${
              selectedPrinter === printer.name
                ? 'border-primary bg-primary-light'
                : 'border-border bg-surface hover:border-primary/30'
            }`}
          >
            <div className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${selectedPrinter === printer.name ? 'bg-primary text-white' : 'bg-background text-text-secondary'}`}>
              <Printer size={20} />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-medium text-text-primary">{printer.name}</span>
                {printer.is_default && (
                  <span className="text-xs bg-primary-light text-primary px-2 py-0.5 rounded-full">默认</span>
                )}
              </div>
              <div className="text-xs text-text-secondary mt-0.5">
                状态: {printer.status} · 端口: {printer.port}
              </div>
            </div>
          </button>
        ))}
        {printers.length === 0 && !loading && (
          <div className="text-center py-8 text-text-secondary text-sm">未检测到打印机</div>
        )}
      </div>

      {/* 打印队列 */}
      {currentPrinter && (
        <div className="bg-surface rounded-xl border border-border overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 bg-background border-b border-border">
            <h4 className="font-medium text-text-primary">打印队列</h4>
            <button
              onClick={handleClearQueue}
              disabled={clearing || jobs.length === 0}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-red-500 hover:bg-red-50 rounded-lg transition-colors disabled:opacity-50"
            >
              <Trash2 size={14} />
              {clearing ? '清空中...' : '清空队列'}
            </button>
          </div>

          {jobs.length > 0 ? (
            <div className="max-h-[300px] overflow-auto">
              {jobs.map((job) => (
                <div
                  key={job.id}
                  className="flex items-center gap-3 px-4 py-2.5 border-t border-border"
                >
                  <FileText size={14} className="text-text-secondary shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-text-primary truncate">{job.document}</div>
                    <div className="text-xs text-text-secondary">{job.size}</div>
                  </div>
                  <span className="text-xs text-text-secondary shrink-0">{job.status}</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="px-4 py-6 text-center text-sm text-text-secondary">队列为空</div>
          )}
        </div>
      )}
    </div>
  );
}
