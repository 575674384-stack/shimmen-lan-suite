import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ClipboardList, Plus, Pin, Trash2, Save } from 'lucide-react';
import type { Announcement } from '../../types';

export default function AnnouncementBoard() {
  const [announcements, setAnnouncements] = useState<Announcement[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [editing, setEditing] = useState<Announcement | null>(null);
  const [form, setForm] = useState<Partial<Announcement>>({});

  const load = async () => {
    try {
      const data = await invoke<Announcement[]>('get_announcements');
      setAnnouncements(data);
    } catch (e) {
      console.error('获取公告失败:', e);
    }
  };

  useEffect(() => {
    load();
    const unlisten = listen('network-message', (event: any) => {
      // 只对 announcements 相关的 StateSync 触发刷新
      const msg = event.payload?.message;
      if (msg?.type === 'StateSync' && msg?.payload?.table === 'announcements') {
        load();
      }
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  const handleSave = async () => {
    if (!form.title?.trim()) return;
    try {
      await invoke('save_announcement', {
        announcement: {
          id: editing?.id || crypto.randomUUID(),
          title: form.title,
          content: form.content || '',
          isPinned: form.is_pinned || false,
          createdBy: '',
          updatedAt: Math.floor(Date.now() / 1000),
        }
      });
      setShowAdd(false);
      setEditing(null);
      setForm({});
      await load();
    } catch (e) {
      console.error('保存公告失败:', e);
      alert('保存公告失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除此公告？')) return;
    try {
      await invoke('delete_announcement', { id });
      await load();
    } catch (e) {
      console.error('删除公告失败:', e);
      alert('删除公告失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  return (
    <div className="h-full flex flex-col bg-background">
      <div className="h-14 bg-surface border-b border-border flex items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <ClipboardList size={18} className="text-primary" />
          <h2 className="font-bold text-text-primary">共享公告栏</h2>
        </div>
        <button onClick={() => { setShowAdd(true); setEditing(null); setForm({}); }} className="p-2 bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors">
          <Plus size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-auto p-4 space-y-3">
        {announcements.map((ann) => (
          <div key={ann.id} className={`bg-surface rounded-xl p-4 shadow-sm ${ann.is_pinned ? 'ring-2 ring-yellow-400' : ''}`}>
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                {ann.is_pinned && <Pin size={16} className="text-yellow-500" />}
                <h3 className="font-bold text-text-primary">{ann.title}</h3>
              </div>
              <div className="flex items-center gap-2">
                <button onClick={() => { setEditing(ann); setForm(ann); setShowAdd(true); }} className="text-xs text-text-secondary hover:text-primary">编辑</button>
                <button onClick={() => handleDelete(ann.id)} className="text-text-secondary hover:text-red-500">
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
            <div className="mt-2 text-sm text-text-secondary whitespace-pre-wrap">{ann.content}</div>
            <div className="mt-2 text-xs text-text-secondary">{new Date(ann.updated_at * 1000).toLocaleString('zh-CN')}</div>
          </div>
        ))}
        {announcements.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-text-secondary">
            <div className="text-4xl mb-2">📋</div>
            <p className="text-sm">暂无公告</p>
          </div>
        )}
      </div>

      {showAdd && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
          <div className="bg-surface rounded-2xl p-6 w-[500px] shadow-xl">
            <h3 className="font-bold text-lg text-text-primary mb-4">{editing ? '编辑' : '新增'}公告</h3>
            <div className="space-y-3">
              <input placeholder="标题" value={form.title || ''} onChange={(e) => setForm({ ...form, title: e.target.value })} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <textarea placeholder="内容" value={form.content || ''} onChange={(e) => setForm({ ...form, content: e.target.value })} rows={5} className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20 resize-none" />
              <label className="flex items-center gap-2 text-sm text-text-secondary">
                <input type="checkbox" checked={form.is_pinned || false} onChange={(e) => setForm({ ...form, is_pinned: e.target.checked })} className="rounded" />
                置顶
              </label>
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
