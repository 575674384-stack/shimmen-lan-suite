import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Monitor, Play, Square, Users } from 'lucide-react';

interface ShareSession {
  peer_id: string;
  peer_name: string;
  active: boolean;
}

const fpsOptions = [
  { value: 5, label: '5fps', desc: '节能' },
  { value: 10, label: '10fps', desc: '标准' },
  { value: 15, label: '15fps', desc: '流畅' },
  { value: 20, label: '20fps', desc: '高速' },
];

const resOptions = [
  { value: 720, label: '高清', desc: '1280×720', recommend: '2K屏推荐' },
  { value: 540, label: '标清', desc: '960×540', recommend: '平衡推荐' },
  { value: 450, label: '流畅', desc: '800×450', recommend: '低配推荐' },
];

export default function ScreenSharePage() {
  const [isSharing, setIsSharing] = useState(false);
  const [watchingPeer, setWatchingPeer] = useState<string | null>(null);
  const [currentFrame, setCurrentFrame] = useState<string>('');
  const [sessions, setSessions] = useState<Map<string, ShareSession>>(new Map());
  const [fps, setFps] = useState(10);
  const [resolution, setResolution] = useState(720);
  const frameRef = useRef<HTMLImageElement>(null);

  // 加载自己的 device_id
  const [myId, setMyId] = useState('');
  useEffect(() => {
    invoke<{ device_id: string }>('get_config').then((config) => {
      setMyId(config.device_id);
    }).catch(() => {
      setMyId('');
    });
  }, []);

  useEffect(() => {
    const unlisten = listen('screen-share', (event: any) => {
      const payload = event.payload;
      if (payload && payload.frame && payload.peer_id !== myId) {
        setCurrentFrame(payload.frame);
        setWatchingPeer(payload.peer_id);
        setSessions(prev => {
          const next = new Map(prev);
          next.set(payload.peer_id, {
            peer_id: payload.peer_id,
            peer_name: payload.peer_id.slice(0, 6) + '...',
            active: true,
          });
          return next;
        });
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [myId]);

  const handleStartShare = async () => {
    try {
      await invoke('start_screen_share', { fps, resolution });
      setIsSharing(true);
    } catch (e) {
      console.error('开始演示失败:', e);
      alert('开始演示失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const handleStopShare = async () => {
    try {
      await invoke('stop_screen_share');
      setIsSharing(false);
    } catch (e) {
      console.error('停止演示失败:', e);
      alert('停止演示失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  // 正在观看某人的演示
  if (currentFrame && watchingPeer) {
    return (
      <div className="h-full flex flex-col bg-background">
        <div className="h-12 bg-surface border-b border-border flex items-center justify-between px-4 shrink-0">
          <div className="flex items-center gap-2">
            <Monitor size={16} className="text-primary" />
            <span className="text-sm font-medium text-text-primary">正在观看演示</span>
          </div>
          <button 
            onClick={() => { setCurrentFrame(''); setWatchingPeer(null); }}
            className="text-xs text-text-secondary hover:text-primary px-3 py-1.5 rounded-lg hover:bg-primary-light transition-colors"
          >
            退出观看
          </button>
        </div>
        <div className="flex-1 overflow-auto p-4 flex items-center justify-center bg-black">
          <img 
            ref={frameRef}
            src={currentFrame} 
            alt="屏幕演示" 
            className="max-w-full max-h-full object-contain rounded-lg"
          />
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-background">
      <div className="h-12 bg-surface border-b border-border flex items-center px-4 shrink-0">
        <Monitor size={16} className="text-primary mr-2" />
        <span className="font-medium text-text-primary">屏幕演示</span>
      </div>
      
      <div className="flex-1 overflow-auto p-6">
        {/* 我是演示者 */}
        <div className="bg-surface rounded-2xl p-6 shadow-sm mb-4">
          <h3 className="font-bold text-text-primary mb-4 flex items-center gap-2">
            <Play size={16} />
            我要演示
          </h3>
          
          {!isSharing && (
            <div className="space-y-4 mb-4">
              {/* 分辨率选择 */}
              <div>
                <label className="text-xs text-text-secondary block mb-2">分辨率（影响清晰度）</label>
                <div className="flex gap-2">
                  {resOptions.map((r) => (
                    <button
                      key={r.value}
                      onClick={() => setResolution(r.value)}
                      className={`flex-1 px-3 py-2 rounded-xl text-xs transition-colors ${
                        resolution === r.value
                          ? 'bg-primary text-white'
                          : 'bg-background text-text-secondary hover:bg-primary-light hover:text-primary'
                      }`}
                    >
                      <div className="font-medium">{r.label}</div>
                      <div className="opacity-70 scale-90">{r.desc}</div>
                    </button>
                  ))}
                </div>
              </div>
              
              {/* 帧率选择 */}
              <div>
                <label className="text-xs text-text-secondary block mb-2">帧率（影响流畅度）</label>
                <div className="flex gap-2">
                  {fpsOptions.map((f) => (
                    <button
                      key={f.value}
                      onClick={() => setFps(f.value)}
                      className={`flex-1 px-3 py-2 rounded-xl text-xs transition-colors ${
                        fps === f.value
                          ? 'bg-primary text-white'
                          : 'bg-background text-text-secondary hover:bg-primary-light hover:text-primary'
                      }`}
                    >
                      <div className="font-medium">{f.label}</div>
                      <div className="opacity-70 scale-90">{f.desc}</div>
                    </button>
                  ))}
                </div>
              </div>
              
              <p className="text-xs text-text-secondary">
                当前配置：
                <span className="text-primary font-medium">
                  {resolution === 720 ? '1280×720' : resolution === 540 ? '960×540' : '800×450'} @ {fps}fps
                </span>
                {' · '}
                {resolution >= 720 ? '文字清晰，适合2K屏演示PPT/文档' : resolution >= 540 ? '平衡画质与性能' : '低配机器优先流畅'}
              </p>
            </div>
          )}
          
          {isSharing ? (
            <div className="flex items-center gap-3">
              <div className="w-3 h-3 bg-red-500 rounded-full animate-pulse" />
              <span className="text-sm text-text-secondary">正在演示中...</span>
              <button 
                onClick={handleStopShare}
                className="ml-auto px-4 py-2 bg-red-500 text-white text-sm rounded-xl hover:bg-red-600 transition-colors flex items-center gap-1"
              >
                <Square size={12} /> 停止演示
              </button>
            </div>
          ) : (
            <button 
              onClick={handleStartShare}
              className="px-6 py-3 bg-primary text-white rounded-xl hover:bg-primary-dark transition-colors flex items-center gap-2"
            >
              <Play size={16} /> 开始演示
            </button>
          )}
        </div>

        {/* 观看他人演示 */}
        <div className="bg-surface rounded-2xl p-6 shadow-sm">
          <h3 className="font-bold text-text-primary mb-4 flex items-center gap-2">
            <Users size={16} />
            观看他人演示
          </h3>
          {sessions.size === 0 ? (
            <p className="text-sm text-text-secondary">当前没有人在演示</p>
          ) : (
            <div className="space-y-2">
              {Array.from(sessions.values()).map((session) => (
                <button
                  key={session.peer_id}
                  onClick={() => setWatchingPeer(session.peer_id)}
                  className="w-full flex items-center gap-3 p-3 rounded-xl bg-background hover:shadow-sm transition-all text-left"
                >
                  <div className="w-10 h-10 bg-primary-light rounded-full flex items-center justify-center">
                    <Monitor size={16} className="text-primary" />
                  </div>
                  <div>
                    <div className="text-sm font-medium text-text-primary">{session.peer_name}</div>
                    <div className="text-xs text-text-secondary flex items-center gap-1">
                      <div className="w-1.5 h-1.5 bg-green-500 rounded-full" />
                      正在演示
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
