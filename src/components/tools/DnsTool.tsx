import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Check } from 'lucide-react';

interface NetworkInterface {
  name: string;
  ip: string;
  dns_servers: string[];
}

const PRESET_DNS = [
  { name: '阿里云', primary: '223.5.5.5', secondary: '223.6.6.6' },
  { name: '腾讯云', primary: '119.29.29.29', secondary: '182.254.116.116' },
  { name: '114', primary: '114.114.114.114', secondary: '114.114.115.115' },
  { name: 'Google', primary: '8.8.8.8', secondary: '8.8.4.4' },
  { name: 'Cloudflare', primary: '1.1.1.1', secondary: '1.0.0.1' },
];

export default function DnsTool() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [selectedInterface, setSelectedInterface] = useState('');
  const [primary, setPrimary] = useState('');
  const [secondary, setSecondary] = useState('');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const loadInterfaces = async () => {
    try {
      const data = await invoke<NetworkInterface[]>('get_network_interfaces');
      setInterfaces(data);
      if (data.length > 0 && !selectedInterface) {
        setSelectedInterface(data[0].name);
      }
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadInterfaces();
  }, []);

  const handleApply = async () => {
    if (!selectedInterface || !primary) return;
    setLoading(true);
    setMessage('');
    try {
      await invoke('set_dns', {
        interface: selectedInterface,
        primary,
        secondary: secondary || null,
      });
      setMessage('DNS 设置成功！');
      loadInterfaces();
    } catch (e) {
      setMessage('设置失败: ' + String(e));
    }
    setLoading(false);
  };

  const currentInterface = interfaces.find((i) => i.name === selectedInterface);

  return (
    <div className="max-w-2xl mx-auto space-y-5">
      <div className="space-y-4">
        <div>
          <label className="block text-sm text-text-secondary mb-2">选择网卡</label>
          <select
            value={selectedInterface}
            onChange={(e) => setSelectedInterface(e.target.value)}
            className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20"
          >
            {interfaces.map((iface) => (
              <option key={iface.name} value={iface.name}>
                {iface.name} {iface.ip ? `(${iface.ip})` : ''}
              </option>
            ))}
          </select>
        </div>

        {currentInterface && (
          <div className="bg-surface rounded-xl p-4 border border-border">
            <div className="text-sm text-text-secondary mb-1">当前 DNS</div>
            <div className="flex flex-wrap gap-2">
              {currentInterface.dns_servers.length > 0 ? (
                currentInterface.dns_servers.map((dns, i) => (
                  <span key={i} className="text-sm bg-background px-3 py-1 rounded-lg text-text-primary">
                    {dns}
                  </span>
                ))
              ) : (
                <span className="text-sm text-text-secondary">自动获取 (DHCP)</span>
              )}
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-sm text-text-secondary mb-2">首选 DNS</label>
            <input
              type="text"
              value={primary}
              onChange={(e) => setPrimary(e.target.value)}
              placeholder="例如 223.5.5.5"
              className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
          </div>
          <div>
            <label className="block text-sm text-text-secondary mb-2">备用 DNS</label>
            <input
              type="text"
              value={secondary}
              onChange={(e) => setSecondary(e.target.value)}
              placeholder="例如 223.6.6.6"
              className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm text-text-secondary mb-2">快速选择</label>
          <div className="flex flex-wrap gap-2">
            {PRESET_DNS.map((dns) => (
              <button
                key={dns.name}
                onClick={() => {
                  setPrimary(dns.primary);
                  setSecondary(dns.secondary);
                }}
                className="px-3 py-1.5 text-sm bg-surface border border-border rounded-lg hover:border-primary hover:text-primary transition-colors"
              >
                {dns.name}
              </button>
            ))}
          </div>
        </div>

        <button
          onClick={handleApply}
          disabled={loading || !primary}
          className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary text-white text-base rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <Check size={18} />
          {loading ? '设置中...' : '应用 DNS'}
        </button>

        {message && (
          <div className={`text-sm text-center py-2 rounded-lg ${message.includes('失败') ? 'text-red-500 bg-red-50' : 'text-green-600 bg-green-50'}`}>
            {message}
          </div>
        )}
      </div>
    </div>
  );
}
