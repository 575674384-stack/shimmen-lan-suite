import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Download, X, Loader2, CheckCircle2, AlertCircle } from 'lucide-react';

interface UpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  download_url: string;
  release_notes: string;
}

type UpdateState = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

export default function UpdatePrompt() {
  const [state, setState] = useState<UpdateState>('idle');
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [errorMsg, setErrorMsg] = useState('');
  const [dismissed, setDismissed] = useState(false);

  const checkUpdate = useCallback(async () => {
    setState('checking');
    try {
      const result = await invoke<UpdateInfo>('check_update');
      if (result.has_update) {
        setInfo(result);
        setState('available');
      } else {
        setState('idle');
      }
    } catch (e) {
      console.error('检查更新失败:', e);
      setState('idle');
    }
  }, []);

  useEffect(() => {
    // 启动 15 秒后自动检查（仅在 auto_update 开启时）
    const timer = setTimeout(async () => {
      try {
        const config = await invoke<{ auto_update: boolean }>('get_config');
        if (config.auto_update !== false) {
          checkUpdate();
        }
      } catch {
        // 读取配置失败时默认检查
        checkUpdate();
      }
    }, 15000);

    // 监听更新就绪事件
    const unlistenReady = listen('update-ready', () => {
      setState('ready');
    });

    // 监听更新错误事件
    const unlistenError = listen<string>('update-error', (event) => {
      setErrorMsg(event.payload);
      setState('error');
    });

    return () => {
      clearTimeout(timer);
      unlistenReady.then((f) => f());
      unlistenError.then((f) => f());
    };
  }, [checkUpdate]);

  const handleUpdate = async () => {
    if (!info?.download_url) return;
    setState('downloading');
    try {
      await invoke('download_and_install', { download_url: info.download_url });
      // 命令返回后下载线程已在后台运行
    } catch (e) {
      setErrorMsg(String(e));
      setState('error');
    }
  };

  const handleDismiss = () => {
    setDismissed(true);
    setState('idle');
  };

  const handleRetry = () => {
    setErrorMsg('');
    setState('idle');
    checkUpdate();
  };

  if (dismissed && state !== 'ready' && state !== 'error') return null;

  // 根据状态渲染不同的 UI
  const renderContent = () => {
    switch (state) {
      case 'available':
        return (
          <div className="flex items-start gap-3">
            <Download className="w-5 h-5 text-primary mt-0.5 shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-foreground">
                发现新版本 v{info?.latest_version}
              </div>
              <div className="text-xs text-muted-foreground mt-1 line-clamp-2">
                当前版本 v{info?.current_version}
              </div>
              <div className="flex gap-2 mt-3">
                <button
                  onClick={handleUpdate}
                  className="px-3 py-1.5 bg-primary text-primary-foreground text-xs rounded-lg hover:bg-primary/90 transition-colors"
                >
                  立即更新
                </button>
                <button
                  onClick={handleDismiss}
                  className="px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  忽略
                </button>
              </div>
            </div>
            <button onClick={handleDismiss} className="text-muted-foreground hover:text-foreground shrink-0">
              <X className="w-4 h-4" />
            </button>
          </div>
        );

      case 'downloading':
        return (
          <div className="flex items-center gap-3">
            <Loader2 className="w-5 h-5 text-primary animate-spin shrink-0" />
            <div className="flex-1">
              <div className="text-sm font-medium text-foreground">正在下载更新...</div>
              <div className="text-xs text-muted-foreground mt-0.5">请保持网络畅通，下载完成后将自动安装</div>
            </div>
          </div>
        );

      case 'ready':
        return (
          <div className="flex items-start gap-3">
            <CheckCircle2 className="w-5 h-5 text-green-500 mt-0.5 shrink-0" />
            <div className="flex-1">
              <div className="text-sm font-medium text-foreground">更新准备就绪</div>
              <div className="text-xs text-muted-foreground mt-1">安装程序已下载，应用即将关闭并启动安装</div>
            </div>
          </div>
        );

      case 'error':
        return (
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-red-500 mt-0.5 shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-foreground">更新失败</div>
              <div className="text-xs text-red-400 mt-1 line-clamp-2">{errorMsg}</div>
              <div className="flex gap-2 mt-3">
                <button
                  onClick={handleRetry}
                  className="px-3 py-1.5 bg-primary text-primary-foreground text-xs rounded-lg hover:bg-primary/90 transition-colors"
                >
                  重试
                </button>
                <button
                  onClick={handleDismiss}
                  className="px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  忽略
                </button>
              </div>
            </div>
            <button onClick={handleDismiss} className="text-muted-foreground hover:text-foreground shrink-0">
              <X className="w-4 h-4" />
            </button>
          </div>
        );

      default:
        return null;
    }
  };

  const content = renderContent();
  if (!content) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 animate-in slide-in-from-bottom-2 fade-in duration-300">
      <div className="bg-card/95 backdrop-blur-sm border border-border rounded-xl shadow-lg p-4">
        {content}
      </div>
    </div>
  );
}
