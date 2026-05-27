import { useState } from 'react';
import type { ChatMessage } from '../../types';
import { Eye } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import FilePreviewModal from './FilePreviewModal';

interface ChatMessageProps {
  message: ChatMessage;
  isSelf: boolean;
  myAvatarBase64?: string;
  myAvatarPreset?: string;
}

const presetMap: Record<string, { emoji: string; gradient: string }> = {
  cat: { emoji: '🐱', gradient: 'from-orange-400 to-pink-500' },
  dog: { emoji: '🐶', gradient: 'from-blue-400 to-purple-500' },
  fox: { emoji: '🦊', gradient: 'from-red-400 to-yellow-500' },
  panda: { emoji: '🐼', gradient: 'from-gray-700 to-gray-300' },
  rabbit: { emoji: '🐰', gradient: 'from-pink-300 to-white' },
  tiger: { emoji: '🐯', gradient: 'from-orange-500 to-yellow-400' },
  frog: { emoji: '🐸', gradient: 'from-green-400 to-emerald-600' },
  octopus: { emoji: '🐙', gradient: 'from-purple-500 to-red-500' },
};

export default function ChatMessageBubble({ message, isSelf, myAvatarBase64, myAvatarPreset }: ChatMessageProps) {
  const [previewPath, setPreviewPath] = useState('');
  const [previewName, setPreviewName] = useState('');

  const time = new Date(message.timestamp * 1000).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });

  const renderAvatar = () => {
    if (isSelf) {
      if (myAvatarBase64) {
        return <img src={myAvatarBase64} alt="me" className="w-full h-full object-cover rounded-full" />;
      }
      if (myAvatarPreset && presetMap[myAvatarPreset]) {
        const p = presetMap[myAvatarPreset];
        return (
          <div className={`w-full h-full bg-gradient-to-br ${p.gradient} rounded-full flex items-center justify-center text-base`}>
            {p.emoji}
          </div>
        );
      }
    }
    return (
      <div className="w-full h-full bg-gradient-to-br from-sky-400 to-blue-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
        {message.sender_name.slice(0, 2)}
      </div>
    );
  };

  const isPreviewableFile = (name: string) => {
    const lower = name.toLowerCase();
    return lower.endsWith('.pdf') || lower.endsWith('.docx');
  };

  const handlePreview = async (filePathOrName: string) => {
    let fullPath = filePathOrName;
    if (!filePathOrName.includes('/') && !filePathOrName.includes('\\')) {
      try {
        const downloadDir = await invoke<string>('get_app_download_dir');
        fullPath = downloadDir + '/' + filePathOrName;
      } catch {
        // ignore
      }
    }
    setPreviewPath(fullPath);
    setPreviewName(filePathOrName.split(/[/\\]/).pop() || filePathOrName);
  };

  return (
    <>
      <div className={`flex items-start gap-2.5 ${isSelf ? 'flex-row-reverse' : ''}`}>
        <div className="w-10 h-10 rounded-full overflow-hidden shrink-0">
          {renderAvatar()}
        </div>
        <div className={`max-w-[70%] ${isSelf ? 'items-end' : 'items-start'} flex flex-col`}>
          <div className={`text-sm text-text-secondary mb-1 ${isSelf ? 'text-right' : ''}`}>
            {message.sender_name} · {time}
          </div>
          <div
            className={`px-4 py-3 rounded-2xl text-base ${
              isSelf
                ? 'bg-primary text-white rounded-tr-md'
                : 'bg-surface text-text-primary rounded-tl-md shadow-sm'
            }`}
          >
            {message.type === 'file' && (
              <div className="flex items-center gap-2">
                <span>📎</span>
                <span className="underline cursor-pointer">{message.content}</span>
                {isPreviewableFile(message.content) && (
                  <button
                    onClick={() => handlePreview(message.content)}
                    className="flex items-center gap-1 px-2 py-0.5 text-xs bg-primary/10 text-primary rounded-md hover:bg-primary/20 transition-colors ml-1"
                  >
                    <Eye size={12} />
                    预览
                  </button>
                )}
              </div>
            )}
            {message.type === 'image' && (
              <div className="rounded-lg overflow-hidden max-w-[240px]">
                {message.content.startsWith('data:image') || message.content.startsWith('http') ? (
                  <img
                    src={message.content}
                    alt="图片"
                    className="max-w-full h-auto rounded-lg cursor-pointer"
                    onClick={() => window.open(message.content, '_blank')}
                  />
                ) : (
                  <span className="text-sm opacity-75">[图片] {message.content}</span>
                )}
              </div>
            )}
            {message.type === 'text' && message.content}
          </div>
        </div>
      </div>

      {previewPath && (
        <FilePreviewModal
          filePath={previewPath}
          fileName={previewName}
          onClose={() => {
            setPreviewPath('');
            setPreviewName('');
          }}
        />
      )}
    </>
  );
}
