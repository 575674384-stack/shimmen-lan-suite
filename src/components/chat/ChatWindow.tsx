import { useState, useRef, useEffect } from 'react';
import { Send, Image, Trash2, FileUp } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useChat } from '../../hooks/useChat';
import { useAvatar } from '../../hooks/useAvatar';
import ChatMessageBubble from './ChatMessage';

export default function ChatWindow() {
  const { messages, myId, sendMessage, clearScreen } = useChat();
  const { avatarBase64, avatarPreset } = useAvatar();
  const [input, setInput] = useState('');
  const [isDragging, setIsDragging] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragCounter = useRef(0);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim()) return;
    const trimmed = input.trim();

    try {
      await sendMessage(trimmed, 'text');
      setInput('');
    } catch (err) {
      console.error('发送消息失败:', err);
      alert('发送失败: ' + (err instanceof Error ? err.message : String(err)));
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSendImage = async () => {
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = 'image/*';
    fileInput.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = async (event) => {
        const base64 = event.target?.result as string;
        await sendMessage(base64, 'image');
      };
      reader.readAsDataURL(file);
    };
    fileInput.click();
  };

  const handleSendFile = async (filePath: string) => {
    try {
      await invoke('send_chat_file', { file_path: filePath });
    } catch (err) {
      alert('发送文件失败: ' + (err instanceof Error ? err.message : String(err)));
    }
  };

  // 拖拽处理
  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current++;
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current--;
    if (dragCounter.current <= 0) {
      dragCounter.current = 0;
      setIsDragging(false);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current = 0;
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length === 0) return;

    for (const file of files) {
      if (file.type.startsWith('image/')) {
        const reader = new FileReader();
        reader.onload = async (event) => {
          const base64 = event.target?.result as string;
          await sendMessage(base64, 'image');
        };
        reader.readAsDataURL(file);
      } else {
        alert('暂不支持拖拽发送此文件，请使用发送文件按钮: ' + file.name);
      }
    }
  };

  return (
    <div className="h-full flex flex-col bg-background">
      <div className="h-16 bg-surface border-b border-border flex items-center justify-between px-5 shrink-0">
        <div className="flex items-center gap-2.5">
          <h2 className="font-bold text-base text-text-primary">共享聊天窗</h2>
          <span className="text-sm text-text-secondary bg-background px-2.5 py-1 rounded-full">全员可见</span>
        </div>
        <button
          onClick={clearScreen}
          className="flex items-center gap-1.5 px-4 py-2 text-sm text-red-500 hover:bg-red-50 rounded-lg transition-colors"
        >
          <Trash2 size={16} />
          清屏
        </button>
      </div>

      <div
        ref={scrollRef}
        className={`flex-1 min-h-0 overflow-auto p-5 space-y-4 scrollbar-hide relative ${
          isDragging ? 'bg-primary/5' : ''
        }`}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        {isDragging && (
          <div className="absolute inset-4 border-2 border-dashed border-primary rounded-2xl bg-primary/5 flex items-center justify-center z-10 pointer-events-none">
            <div className="text-primary text-lg font-medium">松开即可发送文件</div>
          </div>
        )}
        {messages.length === 0 && !isDragging && (
          <div className="flex flex-col items-center justify-center h-full text-text-secondary">
            <div className="text-5xl mb-3">💬</div>
            <p className="text-base">暂无消息，发送一条吧</p>
            <p className="text-sm text-text-secondary mt-2">拖拽文件到此处可直接发送</p>
          </div>
        )}
        {messages.map((msg) => (
          <ChatMessageBubble
            key={msg.id}
            message={msg}
            isSelf={msg.sender_id === myId}
            myAvatarBase64={avatarBase64}
            myAvatarPreset={avatarPreset}
          />
        ))}
      </div>

      <div className="p-4 shrink-0 bg-surface border-t border-border">
        <div className="flex items-center gap-3">
          <button
            onClick={async () => {
              const filePath = await invoke<string | null>('select_file');
              if (!filePath) return;
              await handleSendFile(filePath);
            }}
            className="p-2.5 text-text-secondary hover:text-primary hover:bg-primary-light rounded-xl transition-colors"
            title="发送文件"
          >
            <FileUp size={22} />
          </button>
          <button
            onClick={handleSendImage}
            className="p-2.5 text-text-secondary hover:text-primary hover:bg-primary-light rounded-xl transition-colors"
            title="发送图片"
          >
            <Image size={22} />
          </button>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入消息..."
            className="flex-1 px-4 py-2.5 bg-background border-0 rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20 placeholder:text-text-secondary"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim()}
            className="p-2.5 bg-primary text-white rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send size={20} />
          </button>
        </div>
      </div>
    </div>
  );
}
