import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Key, Plus, Eye, EyeOff, Trash2, Copy, Save } from 'lucide-react';
import type { PasswordEntry } from '../../types';

export default function PasswordVault() {
  const [entries, setEntries] = useState<PasswordEntry[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [editing, setEditing] = useState<PasswordEntry | null>(null);
  const [visibleIds, setVisibleIds] = useState<Set<string>>(new Set());
  const [form, setForm] = useState<Partial<PasswordEntry>>({});

  const load = async () => {
    try {
      const data = await invoke<PasswordEntry[]>('get_passwords');
      setEntries(data);
    } catch (e) {
      console.error('获取密码失败:', e);
    }
  };

  useEffect(() => {
    load();
    const unlisten = listen('password-saved', () => {
      load();
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  const handleSave = async () => {
    if (!form.name?.trim()) return;
    try {
      await invoke('save_password', {
        entry: {
          id: editing?.id || crypto.randomUUID(),
          name: form.name,
          account: form.account || '',
          password: form.password || '',
          note: form.note || '',
          createdBy: '',
          updatedAt: Math.floor(Date.now() / 1000),
        }
      });
      setShowAdd(false);
      setEditing(null);
      setForm({});
      await load();
    } catch (e) {
      console.error('保存密码失败:', e);
      alert('保存密码失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除此密码条目？')) return;
    try {
      await invoke('delete_password', { id });
      await load();
    } catch (e) {
      console.error('删除密码失败:', e);
      alert('删除密码失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const toggleVisible = (id: string) => {
    setVisibleIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div className="h-full flex flex-col bg-background">
      <div className="h-14 bg-surface border-b border-border flex items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <Key size={18} className="text-primary" />
          <h2 className="font-bold text-text-primary">共享密码栏</h2>
        </div>
        <button onClick={() => { setShowAdd(true); setEditing(null); setForm({}); }} className="p-2 bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors">
          <Plus size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-auto p-4">
        <div className="bg-surface rounded-xl shadow-sm overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-background text-text-secondary">
              <tr>
                <th className="text-left px-4 py-3 font-medium">名称</th>
                <th className="text-left px-4 py-3 font-medium">账号</th>
                <th className="text-left px-4 py-3 font-medium">密码</th>
                <th className="text-left px-4 py-3 font-medium">备注</th>
                <th className="text-right px-4 py-3 font-medium">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {entries.map((entry) => (
                <tr key={entry.id} className="hover:bg-background">
                  <td className="px-4 py-3 font-medium text-text-primary">{entry.name}</td>
                  <td className="px-4 py-3 text-text-secondary">{entry.account}</td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-text-primary">
                        {visibleIds.has(entry.id) ? entry.password : '••••••••'}
                      </span>
                      <button onClick={() => toggleVisible(entry.id)} className="text-text-secondary hover:text-primary">
                        {visibleIds.has(entry.id) ? <EyeOff size={14} /> : <Eye size={14} />}
                      </button>
                      <button onClick={() => navigator.clipboard.writeText(entry.password)} className="text-text-secondary hover:text-primary">
                        <Copy size={14} />
                      </button>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-text-secondary">{entry.note}</td>
                  <td className="px-4 py-3 text-right">
                    <button onClick={() => { setEditing(entry); setForm(entry); setShowAdd(true); }} className="text-text-secondary hover:text-primary mr-2">编辑</button>
                    <button onClick={() => handleDelete(entry.id)} className="text-text-secondary hover:text-red-500">
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              ))}
              {entries.length === 0 && (
                <tr><td colSpan={5} className="px-4 py-8 text-center text-text-secondary">暂无密码条目</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {showAdd && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
          <div className="bg-surface rounded-2xl p-6 w-96 shadow-xl">
            <h3 className="font-bold text-lg text-text-primary mb-4">{editing ? '编辑' : '新增'}密码</h3>
            <div className="space-y-3">
              <input placeholder="名称" value={form.name || ''} onChange={(e) => setForm({ ...form, name: e.target.value })} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <input placeholder="账号" value={form.account || ''} onChange={(e) => setForm({ ...form, account: e.target.value })} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <input type="text" placeholder="密码" value={form.password || ''} onChange={(e) => setForm({ ...form, password: e.target.value })} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <input placeholder="备注" value={form.note || ''} onChange={(e) => setForm({ ...form, note: e.target.value })} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20" />
            </div>
            <div className="flex justify-end gap-2 mt-6">
              <button onClick={() => setShowAdd(false)} className="px-4 py-2 text-sm text-text-secondary hover:bg-background rounded-lg">取消</button>
              <button onClick={handleSave} className="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-dark flex items-center gap-1">
                <Save size={14} /> 保存
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
