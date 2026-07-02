import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface LimpezaAvancadaProps {
  onVoltar: () => void;
}

function LimpezaAvancada({ onVoltar }: LimpezaAvancadaProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [excluirSombras, setExcluirSombras] = useState(false);
  const [pending, setPending] = useState<"temp" | "full" | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logLimpeza, addLog);

  async function run() {
    const action = pending;
    setPending(null);
    if (!action) return;
    setBusy(true);
    setLogs([]);
    try {
      const count = await invoke<number>(action === "temp" ? "limpar_temporarios" : "limpeza_completa", {
        excluirSombras,
      });
      addLog(`${count} alvo(s) de limpeza processado(s).`);
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Limpeza Avançada">
      <div className="grid flex-shrink-0 grid-cols-1 gap-4 md:grid-cols-2">
        <section className="space-y-4 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Temporários</h3>
          <button
            type="button"
            disabled={busy}
            onClick={() => setPending("temp")}
            className="w-full rounded-lg bg-accent-orange px-4 py-3 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Limpar Temporários
          </button>
        </section>
        <section className="space-y-4 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Limpeza do PC</h3>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={excluirSombras}
              onChange={(event) => setExcluirSombras(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>Excluir sombras VSS</span>
          </label>
          <button
            type="button"
            disabled={busy}
            onClick={() => setPending("full")}
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Executar Limpeza
          </button>
        </section>
      </div>
      <LogPanel logLines={logs} />
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
      {pending && (
        <Modal
          tipo="aviso"
          titulo={pending === "temp" ? "Limpar arquivos temporários?" : "Executar limpeza completa?"}
          mensagem={
            pending === "temp"
              ? "Esta ação remove conteúdo de diretórios temporários e caches conhecidos após validação de caminho."
              : "Esta ação executa limpeza ampla, cleanmgr e preserva logs MongoDB DigiSat do dia/mais recente. Sombras VSS só serão excluídas se marcado."
          }
          textoConfirmar="Executar"
          textoCancelar="Cancelar"
          onConfirmar={() => void run()}
          onCancelar={() => setPending(null)}
        />
      )}
    </FeatureContainer>
  );
}

export default LimpezaAvancada;
