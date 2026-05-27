import { useState, useEffect } from 'react';
import { X, Send, Paperclip, FolderUp } from 'lucide-react';
import type { User, TransferRecord } from '../../types';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface TransferWindowProps {
  user: User;
  onClose: () => void;
}

export default function TransferWindow({ user, onClose }: TransferWindowProps) {
  const [records, setRecords] = useState<TransferRecord[]>([
    { id: '1', type: 'received', fileName: '项目方案.docx', fileSize: '2.4 MB', progress: 100, status: 'completed' },
    { id: '2', type: 'sent', fileName: '截图_01.png', fileSize: '1.1 MB', progress: 100, status: 'completed' },
  ]);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    const setupListener = async () => {
      const unlisten = await listen('file-received', (event) => {
        const payload = event.payload as { file_name: string; download_path: string; peer_id: string };
        const newRecord: TransferRecord = {
          id: crypto.randomUUID(),
          type: 'received',
          fileName: payload.file_name,
          fileSize: 'Unknown',
          progress: 100,
          status: 'completed',
        };
        setRecords(prev => [...prev, newRecord]);
      });
      unlistenFn = unlisten;
    };
    setupListener();
    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const handleSendFile = async () => {
    const filePath = await invoke<string | null>('select_file');
    if (!filePath) return;

    const fileName = filePath.split(/[/\\]/).pop() || 'unknown';
    const newRecord: TransferRecord = {
      id: crypto.randomUUID(),
      type: 'sent',
      fileName,
      fileSize: '...',
      progress: 0,
      status: 'transferring',
    };
    setRecords(prev => [...prev, newRecord]);

    try {
      await invoke('send_file_to_peer', { peerId: user.id, filePath });
      setRecords(prev => prev.map(r => r.id === newRecord.id ? { ...r, status: 'completed', progress: 100 } : r));
      // 同时在共享聊天中发送文件消息
      await invoke('send_chat_message', {
        content: fileName,
        messageType: 'file',
      });
    } catch (err) {
      console.error('Send file failed:', err);
      setRecords(prev => prev.map(r => r.id === newRecord.id ? { ...r, status: 'failed' } : r));
    }
  };

  return (
    <div className="flex-1 flex flex-col bg-surface">
      <div className="h-14 border-b border-border flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-sky-400 to-blue-500 rounded-full flex items-center justify-center text-white text-xs font-medium">
            {user.username.slice(0, 2)}
          </div>
          <div>
            <div className="font-medium text-sm text-text-primary">{user.username}</div>
            <div className="text-xs text-text-secondary">{user.ip}</div>
          </div>
        </div>
        <button onClick={onClose} className="p-1.5 text-text-secondary hover:text-text-secondary hover:bg-background rounded-lg transition-colors">
          <X size={18} />
        </button>
      </div>
      
      <div className="flex-1 min-h-0 overflow-auto p-4 space-y-3">
        {records.map((record) => (
          <div
            key={record.id}
            className={`flex items-start gap-3 ${record.type === 'sent' ? 'flex-row-reverse' : ''}`}
          >
            <div className={`max-w-[70%] p-3 rounded-2xl ${
              record.type === 'sent'
                ? 'bg-primary text-white rounded-tr-md'
                : 'bg-background text-text-primary rounded-tl-md'
            }`}>
              <div className="flex items-center gap-2 mb-1">
                <FolderUp size={14} className={record.type === 'sent' ? 'text-primary-light' : 'text-text-secondary'} />
                <span className="text-sm font-medium">{record.fileName}</span>
              </div>
              <div className="text-xs opacity-75">{record.fileSize}</div>
              {record.status === 'transferring' && (
                <div className="mt-2 h-1 bg-black/10 rounded-full overflow-hidden">
                  <div className="h-full bg-surface/80 rounded-full transition-all" style={{ width: `${record.progress}%` }} />
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
      
      <div className="p-3 shrink-0 border-t border-border">
        <div className="flex items-center gap-2">
          <button
            onClick={handleSendFile}
            className="p-2 text-text-secondary hover:text-primary hover:bg-primary-light rounded-lg transition-colors"
          >
            <Paperclip size={18} />
          </button>
          <input
            type="text"
            placeholder="拖拽文件到此处，或输入消息..."
            className="flex-1 px-3 py-2 bg-background border-0 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary/20 placeholder:text-text-secondary"
          />
          <button className="p-2 bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors">
            <Send size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
