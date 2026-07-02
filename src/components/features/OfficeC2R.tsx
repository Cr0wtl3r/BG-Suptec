import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface OfficeC2RProps {
  onVoltar: () => void;
}

interface OfficeC2rInstall {
  clientExe: string;
  clickToRunExe: string;
  installRoot: string;
  platform: string;
  culture: string;
  version: string;
  audienceId: string;
}

interface OfficeUpdateChannel {
  id: string;
  name: string;
  ffn: string;
}

type Pending = "channel" | "add" | "remove";

function OfficeC2R({ onVoltar }: OfficeC2RProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [install, setInstall] = useState<OfficeC2rInstall | null>(null);
  const [channels, setChannels] = useState<OfficeUpdateChannel[]>([]);
  const [channelId, setChannelId] = useState("CC");
  const [produtoId, setProdutoId] = useState("ProPlus2024Volume");
  const [appsExcluidos, setAppsExcluidos] = useState("groove,lync,teams");
  const [pending, setPending] = useState<Pending | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logOfficeC2r, addLog);

  useEffect(() => {
    async function load() {
      try {
        setChannels(await invoke<OfficeUpdateChannel[]>("obter_canais_office"));
        setInstall(await invoke<OfficeC2rInstall>("detectar_office_c2r"));
      } catch (err) {
        addLog(`Office Click-to-Run não detectado: ${err}`);
      }
    }
    void load();
  }, []);

  async function run() {
    const action = pending;
    setPending(null);
    if (!action) return;
    setBusy(true);
    setLogs([]);
    try {
      const excluded = appsExcluidos
        .split(",")
        .map((value) => value.trim())
        .filter(Boolean);
      const command =
        action === "channel"
          ? invoke<string>("alterar_canal_office", { canalId: channelId })
          : action === "add"
            ? invoke<string>("adicionar_produto_office", { produtoId, appsExcluidos: excluded })
            : invoke<string>("remover_produto_office", { produtoId });
      addLog(await command);
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Office Click-to-Run">
      <div className="grid flex-shrink-0 grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Instalação</h3>
          <p className="m-0 break-words text-sm opacity-80">
            {install ? install.installRoot : "Click-to-Run não detectado"}
          </p>
          <select
            value={channelId}
            onChange={(event) => setChannelId(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            {channels.map((channel) => (
              <option key={channel.id} value={channel.id}>
                {channel.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={busy || !install}
            onClick={() => setPending("channel")}
            className="w-full rounded-lg bg-accent-orange px-4 py-3 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Alterar Canal
          </button>
        </section>
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Produto</h3>
          <input
            value={produtoId}
            onChange={(event) => setProdutoId(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          />
          <input
            value={appsExcluidos}
            onChange={(event) => setAppsExcluidos(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          />
          <div className="grid grid-cols-2 gap-3">
            <button
              type="button"
              disabled={busy || !install || !produtoId}
              onClick={() => setPending("add")}
              className="rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
            >
              Adicionar
            </button>
            <button
              type="button"
              disabled={busy || !install || !produtoId}
              onClick={() => setPending("remove")}
              className="rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
            >
              Remover
            </button>
          </div>
        </section>
      </div>
      <LogPanel logLines={logs} />
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
      {pending && (
        <Modal
          tipo="aviso"
          titulo="Executar alteração no Office?"
          mensagem="Esta ação usa Office Click-to-Run local. Não executa ativação, Ohook ou scripts MAS."
          textoConfirmar="Executar"
          textoCancelar="Cancelar"
          onConfirmar={() => void run()}
          onCancelar={() => setPending(null)}
        />
      )}
    </FeatureContainer>
  );
}

export default OfficeC2R;
