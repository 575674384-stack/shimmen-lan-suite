import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { KeyRound, AlertTriangle, CheckCircle } from 'lucide-react';

interface ActivateResult {
  success: boolean;
  output: string;
}

const EDITIONS = [
  { id: 'home', name: 'Windows 10 家庭版' },
  { id: 'professional', name: 'Windows 10 专业版' },
  { id: 'enterprise', name: 'Windows 10 企业版' },
];

export default function WinActivateTool() {
  const [edition, setEdition] = useState('professional');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ActivateResult | null>(null);

  const handleActivate = async () => {
    setLoading(true);
    setResult(null);
    try {
      const data = await invoke<ActivateResult>('activate_windows', { edition });
      setResult(data);
    } catch (e) {
      setResult({ success: false, output: String(e) });
    }
    setLoading(false);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-5">
      <div className="bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-start gap-3">
        <AlertTriangle size={20} className="text-amber-500 shrink-0 mt-0.5" />
        <div className="text-sm text-amber-700">
          本工具仅供学习和测试使用。请确保您拥有合法的 Windows 授权。
          激活需要管理员权限，且需联网连接 KMS 服务器。
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label className="block text-sm text-text-secondary mb-2">选择系统版本</label>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            {EDITIONS.map((ed) => (
              <button
                key={ed.id}
                onClick={() => setEdition(ed.id)}
                className={`px-4 py-3 rounded-xl border text-sm font-medium transition-all ${
                  edition === ed.id
                    ? 'border-primary bg-primary-light text-primary'
                    : 'border-border bg-surface text-text-secondary hover:border-primary/30'
                }`}
              >
                {ed.name}
              </button>
            ))}
          </div>
        </div>

        <button
          onClick={handleActivate}
          disabled={loading}
          className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary text-white text-base rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <KeyRound size={18} />
          {loading ? '正在激活...' : '一键激活'}
        </button>

        {result && (
          <div className={`rounded-xl p-4 border ${result.success ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}>
            <div className="flex items-center gap-2 mb-2">
              {result.success ? (
                <CheckCircle size={18} className="text-green-500" />
              ) : (
                <AlertTriangle size={18} className="text-red-500" />
              )}
              <span className={`font-medium ${result.success ? 'text-green-700' : 'text-red-700'}`}>
                {result.success ? '激活命令已执行' : '激活失败'}
              </span>
            </div>
            <pre className="text-xs bg-background p-3 rounded-lg overflow-auto max-h-64 whitespace-pre-wrap font-mono text-text-secondary">
              {result.output}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}
