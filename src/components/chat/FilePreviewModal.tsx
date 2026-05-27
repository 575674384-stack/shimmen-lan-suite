import { useState, useEffect } from 'react';
import { X, FileText, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import mammoth from 'mammoth';

interface FilePreviewModalProps {
  filePath: string;
  fileName: string;
  onClose: () => void;
}

export default function FilePreviewModal({ filePath, fileName, onClose }: FilePreviewModalProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [pdfDataUrl, setPdfDataUrl] = useState('');
  const [docxHtml, setDocxHtml] = useState('');

  const ext = fileName.toLowerCase().split('.').pop() || '';
  const isPdf = ext === 'pdf';
  const isDocx = ext === 'docx';

  useEffect(() => {
    loadPreview();
  }, [filePath]);

  const loadPreview = async () => {
    setLoading(true);
    setError('');
    try {
      const base64 = await invoke<string>('read_file_base64', { file_path: filePath });
      const binaryString = atob(base64);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }

      if (isPdf) {
        const blob = new Blob([bytes], { type: 'application/pdf' });
        const url = URL.createObjectURL(blob);
        setPdfDataUrl(url);
      } else if (isDocx) {
        const result = await mammoth.convertToHtml({ arrayBuffer: bytes.buffer });
        setDocxHtml(result.value);
      } else {
        setError('不支持的文件格式');
      }
    } catch (err) {
      setError('加载预览失败: ' + (err instanceof Error ? err.message : String(err)));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    return () => {
      if (pdfDataUrl) {
        URL.revokeObjectURL(pdfDataUrl);
      }
    };
  }, [pdfDataUrl]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-surface rounded-xl shadow-xl w-full max-w-3xl h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border shrink-0">
          <div className="flex items-center gap-2">
            <FileText size={18} className="text-primary" />
            <span className="font-medium text-sm text-text-primary truncate max-w-[400px]">{fileName}</span>
          </div>
          <button
            onClick={onClose}
            className="p-1 text-text-secondary hover:text-text-primary hover:bg-background rounded-lg transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 min-h-0 overflow-auto p-4 bg-background">
          {loading && (
            <div className="flex items-center justify-center h-full text-text-secondary">
              <Loader2 size={24} className="animate-spin mr-2" />
              <span className="text-sm">正在加载预览...</span>
            </div>
          )}

          {error && (
            <div className="flex items-center justify-center h-full text-red-500 text-sm">
              {error}
            </div>
          )}

          {isPdf && pdfDataUrl && (
            <iframe
              src={pdfDataUrl}
              className="w-full h-full rounded-lg border border-border"
              title={fileName}
            />
          )}

          {isDocx && docxHtml && (
            <div
              className="prose prose-sm max-w-none bg-white rounded-lg border border-border p-6"
              dangerouslySetInnerHTML={{ __html: docxHtml }}
            />
          )}
        </div>
      </div>
    </div>
  );
}
