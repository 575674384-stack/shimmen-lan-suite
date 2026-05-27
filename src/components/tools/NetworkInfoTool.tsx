import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Network, Server, RefreshCw, CheckCircle, XCircle } from 'lucide-react';

interface NetworkDetail {
  adapter_name: string;
  mac: string;
  ip: string;
  subnet_mask: string;
  gateway: string;
  dns_servers: string[];
  dhcp_enabled: boolean;
}

interface NetworkStatus {
  internet_connected: boolean;
  lan_connected: boolean;
  public_ip: string;
}

export default function NetworkInfoTool() {
  const [details, setDetails] = useState<NetworkDetail[]>([]);
  const [status, setStatus] = useState<NetworkStatus | null>(null);
  const [loading, setLoading] = useState(false);

  const loadData = async () => {
    setLoading(true);
    try {
      const [d, s] = await Promise.all([
        invoke<NetworkDetail[]>('get_network_details'),
        invoke<NetworkStatus>('get_network_status'),
      ]);
      setDetails(d);
      setStatus(s);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-lg text-text-primary">本机网络信息</h3>
        <button
          onClick={loadData}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      {/* 连通状态 */}
      {status && (
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <StatusCard
            label="互联网"
            connected={status.internet_connected}
          />
          <StatusCard
            label="局域网"
            connected={status.lan_connected}
          />
          <div className="bg-surface rounded-xl p-4 border border-border flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center shrink-0">
              <Server size={20} className="text-blue-600" />
            </div>
            <div className="min-w-0">
              <div className="text-xs text-text-secondary">公网 IP</div>
              <div className="text-sm text-text-primary font-medium truncate">{status.public_ip}</div>
            </div>
          </div>
        </div>
      )}

      {/* 网卡详情 */}
      <div className="space-y-3">
        <h4 className="font-medium text-text-primary">网卡详情</h4>
        {details.map((detail, i) => (
          <div key={i} className="bg-surface rounded-xl p-4 border border-border space-y-3">
            <div className="flex items-center gap-2">
              <Network size={16} className="text-primary" />
              <span className="font-medium text-text-primary">{detail.adapter_name}</span>
              {detail.dhcp_enabled && (
                <span className="text-xs bg-amber-100 text-amber-700 px-2 py-0.5 rounded-full">DHCP</span>
              )}
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm">
              <InfoRow label="IP 地址" value={detail.ip || '-'} />
              <InfoRow label="MAC 地址" value={detail.mac || '-'} />
              <InfoRow label="子网掩码" value={detail.subnet_mask || '-'} />
              <InfoRow label="默认网关" value={detail.gateway || '-'} />
              <div className="sm:col-span-2">
                <span className="text-text-secondary">DNS: </span>
                <span className="text-text-primary">
                  {detail.dns_servers.length > 0 ? detail.dns_servers.join(', ') : '-'}
                </span>
              </div>
            </div>
          </div>
        ))}
        {details.length === 0 && !loading && (
          <div className="text-center py-8 text-text-secondary text-sm">未检测到活动网卡</div>
        )}
      </div>
    </div>
  );
}

function StatusCard({ label, connected }: { label: string; connected: boolean }) {
  return (
    <div className="bg-surface rounded-xl p-4 border border-border flex items-center gap-3">
      <div className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${connected ? 'bg-green-100' : 'bg-red-100'}`}>
        {connected ? (
          <CheckCircle size={20} className="text-green-600" />
        ) : (
          <XCircle size={20} className="text-red-600" />
        )}
      </div>
      <div>
        <div className="text-xs text-text-secondary">{label}</div>
        <div className={`text-sm font-medium ${connected ? 'text-green-600' : 'text-red-600'}`}>
          {connected ? '已连通' : '未连通'}
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="text-text-secondary">{label}: </span>
      <span className="text-text-primary">{value}</span>
    </div>
  );
}
