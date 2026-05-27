import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen, FileEdit, Eye, Check, AlertTriangle } from 'lucide-react';

interface FileEntry {
  name: string;
  is_dir: boolean;
}

interface RenamePreview {
  old_name: string;
  new_name: string;
}

interface RenameResult {
  old_name: string;
  new_name: string;
  success: boolean;
  error?: string;
}

const RULE_TYPES = [
  { id: 'add_prefix', name: '添加前缀', desc: '在文件名前添加指定文字' },
  { id: 'add_suffix', name: '添加后缀', desc: '在文件名（不含扩展名）后添加文字' },
  { id: 'replace', name: '替换文本', desc: '将文件名中的某段文字替换为另一段' },
  { id: 'numbering', name: '序号重命名', desc: '按序号规则重新命名' },
  { id: 'change_ext', name: '修改扩展名', desc: '批量修改文件后缀' },
];

export default function BatchRenameTool() {
  const [path, setPath] = useState('');
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [ruleType, setRuleType] = useState('add_prefix');
  const [prefix, setPrefix] = useState('');
  const [suffix, setSuffix] = useState('');
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [startNum, setStartNum] = useState(1);
  const [digits, setDigits] = useState(3);
  const [newExt, setNewExt] = useState('');
  const [preview, setPreview] = useState<RenamePreview[]>([]);
  const [results, setResults] = useState<RenameResult[]>([]);
  const [loading, setLoading] = useState(false);

  const selectFolder = async () => {
    try {
      const selected = await invoke<string | null>('select_folder');
      if (selected) {
        setPath(selected);
        loadFiles(selected);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const loadFiles = async (dir: string) => {
    try {
      const data = await invoke<FileEntry[]>('list_files_in_dir', { path: dir });
      setFiles(data);
      setPreview([]);
      setResults([]);
    } catch (e) {
      console.error(e);
    }
  };

  const buildRule = () => {
    return {
      rule_type: ruleType,
      prefix: prefix || null,
      suffix: suffix || null,
      from: from || null,
      to: to || null,
      start: startNum,
      digits: digits,
      new_ext: newExt || null,
    };
  };

  const handlePreview = async () => {
    if (!path) return;
    setLoading(true);
    try {
      const data = await invoke<RenamePreview[]>('preview_rename', {
        path,
        rule: buildRule(),
      });
      setPreview(data);
      setResults([]);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const handleExecute = async () => {
    if (!path) return;
    setLoading(true);
    try {
      const data = await invoke<RenameResult[]>('execute_rename', {
        path,
        rule: buildRule(),
      });
      setResults(data);
      setPreview([]);
      loadFiles(path);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const fileCount = files.filter((f) => !f.is_dir).length;

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      {/* 选择目录 */}
      <div className="flex items-center gap-3">
        <button
          onClick={selectFolder}
          className="flex items-center gap-2 px-4 py-2.5 bg-surface border border-border rounded-xl text-sm hover:border-primary transition-colors"
        >
          <FolderOpen size={16} />
          {path ? '更换目录' : '选择目录'}
        </button>
        {path && (
          <span className="text-sm text-text-secondary truncate">{path}</span>
        )}
      </div>

      {path && (
        <div className="text-sm text-text-secondary">
          该目录共有 <span className="text-text-primary font-medium">{fileCount}</span> 个文件
        </div>
      )}

      {/* 规则选择 */}
      <div className="space-y-3">
        <label className="block text-sm text-text-secondary">重命名规则</label>
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
          {RULE_TYPES.map((rt) => (
            <button
              key={rt.id}
              onClick={() => { setRuleType(rt.id); setPreview([]); setResults([]); }}
              className={`px-3 py-2 rounded-xl border text-xs font-medium transition-all ${
                ruleType === rt.id
                  ? 'border-primary bg-primary-light text-primary'
                  : 'border-border bg-surface text-text-secondary hover:border-primary/30'
              }`}
            >
              {rt.name}
            </button>
          ))}
        </div>

        {/* 规则参数 */}
        <div className="bg-surface rounded-xl p-4 border border-border space-y-3">
          {ruleType === 'add_prefix' && (
            <div>
              <label className="block text-sm text-text-secondary mb-1.5">前缀内容</label>
              <input
                value={prefix}
                onChange={(e) => setPrefix(e.target.value)}
                placeholder="例如：2024-"
                className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
          )}

          {ruleType === 'add_suffix' && (
            <div>
              <label className="block text-sm text-text-secondary mb-1.5">后缀内容</label>
              <input
                value={suffix}
                onChange={(e) => setSuffix(e.target.value)}
                placeholder="例如：_backup"
                className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
          )}

          {ruleType === 'replace' && (
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">查找</label>
                <input
                  value={from}
                  onChange={(e) => setFrom(e.target.value)}
                  placeholder="原文字"
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">替换为</label>
                <input
                  value={to}
                  onChange={(e) => setTo(e.target.value)}
                  placeholder="新文字"
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
            </div>
          )}

          {ruleType === 'numbering' && (
            <div className="grid grid-cols-3 gap-3">
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">前缀</label>
                <input
                  value={prefix}
                  onChange={(e) => setPrefix(e.target.value)}
                  placeholder="img_"
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">起始序号</label>
                <input
                  type="number"
                  value={startNum}
                  onChange={(e) => setStartNum(Number(e.target.value))}
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">位数</label>
                <input
                  type="number"
                  value={digits}
                  onChange={(e) => setDigits(Number(e.target.value))}
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
            </div>
          )}

          {ruleType === 'change_ext' && (
            <div>
              <label className="block text-sm text-text-secondary mb-1.5">新扩展名（不含点）</label>
              <input
                value={newExt}
                onChange={(e) => setNewExt(e.target.value)}
                placeholder="例如：txt"
                className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
          )}
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-3">
        <button
          onClick={handlePreview}
          disabled={loading || !path}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-surface border border-border text-text-primary rounded-xl hover:bg-background transition-colors disabled:opacity-50"
        >
          <Eye size={16} />
          预览
        </button>
        <button
          onClick={handleExecute}
          disabled={loading || !path}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-primary text-white rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <FileEdit size={16} />
          执行重命名
        </button>
      </div>

      {/* 预览 */}
      {preview.length > 0 && (
        <div className="bg-surface rounded-xl border border-border overflow-hidden">
          <div className="px-4 py-2 bg-background text-xs text-text-secondary font-medium">预览</div>
          <div className="max-h-[300px] overflow-auto">
            {preview.map((p, i) => (
              <div key={i} className="grid grid-cols-2 gap-4 px-4 py-2 border-t border-border text-sm">
                <span className="text-text-secondary truncate" title={p.old_name}>{p.old_name}</span>
                <span className="text-primary truncate" title={p.new_name}>{p.new_name}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 执行结果 */}
      {results.length > 0 && (
        <div className="space-y-2">
          <h4 className="font-medium text-text-primary">执行结果</h4>
          {results.map((r, i) => (
            <div
              key={i}
              className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${r.success ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}`}
            >
              {r.success ? <Check size={14} /> : <AlertTriangle size={14} />}
              <span className="truncate">{r.old_name} → {r.new_name}</span>
              {r.error && <span className="text-xs opacity-70">({r.error})</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
