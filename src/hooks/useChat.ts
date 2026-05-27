import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ChatMessage } from '../types';

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [myId, setMyId] = useState('');

  // 加载自己的 device_id
  useEffect(() => {
    invoke<{ device_id: string }>('get_config').then((config) => {
      setMyId(config.device_id);
    }).catch(() => {
      setMyId('');
    });
  }, []);

  // 加载历史记录
  useEffect(() => {
    const loadHistory = async () => {
      try {
        const history = await invoke<Array<{
          id: string;
          sender_id: string;
          sender_name: string;
          message_type: string;
          content: string;
          timestamp: number;
        }>>('get_chat_history');
        const formatted = history.map((msg) => ({
          id: msg.id,
          sender_id: msg.sender_id,
          sender_name: msg.sender_name,
          type: msg.message_type as 'text' | 'file' | 'image',
          content: msg.content,
          timestamp: msg.timestamp * 1000,
        }));
        setMessages(formatted);
      } catch (e) {
        console.error('加载聊天记录失败:', e);
      }
    };
    loadHistory();
  }, []);

  useEffect(() => {
    const unlisten = listen('network-message', (event: any) => {
      const payload = event.payload;
      if (payload && payload.message) {
        const msg = payload.message;
        if (msg.type === 'ChatMessage') {
          const chatMsg: ChatMessage = {
            id: msg.payload.id,
            sender_id: msg.payload.sender_id,
            sender_name: msg.payload.sender_name,
            type: msg.payload.message_type,
            content: msg.payload.content,
            timestamp: Date.now(),
          };
          setMessages((prev) => [...prev, chatMsg]);
        } else if (msg.type === 'ClearScreen') {
          setMessages([]);
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 监听文件接收事件，自动在聊天中显示文件消息
  useEffect(() => {
    const unlisten = listen('file-received', (event: any) => {
      const payload = event.payload as { file_name: string; download_path: string; peer_id: string };
      const chatMsg: ChatMessage = {
        id: crypto.randomUUID(),
        sender_id: payload.peer_id,
        sender_name: '文件传输',
        type: 'file',
        content: payload.download_path,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, chatMsg]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const sendMessage = useCallback(async (content: string, type: 'text' | 'file' | 'image' = 'text') => {
    await invoke('send_chat_message', {
      content,
      messageType: type,
    });
  }, []);

  const clearScreen = useCallback(async () => {
    await invoke('clear_chat_screen');
    setMessages([]);
  }, []);

  return { messages, myId, sendMessage, clearScreen };
}
