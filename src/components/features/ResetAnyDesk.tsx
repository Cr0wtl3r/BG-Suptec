import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface ResetAnyDeskProps {
  onVoltar: () => void;
}

function ResetAnyDesk({ onVoltar }: ResetAnyDeskProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logAnydesk, addLog);

  async function run() {
    setConfirming(false);
    setBusy(true);
    setLogs([]);
    try {
      await invoke("resetar_anydesk");
      addLog("Reset AnyDesk concluído.");
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Reset AnyDesk">
      <div className="mx-auto w-full max-w-xl flex-shrink-0">
        <button
          type="button"
          disabled={busy}
          onClick={() => setConfirming(true)}
          className="w-full rounded-lg bg-accent-orange px-4 py-4 text-lg font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
        >
          Resetar AnyDesk
        </button>
      </div>
      <LogPanel logLines={logs} />
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
      {confirming && (
        <Modal
          tipo="aviso"
          titulo="Resetar AnyDesk?"
          mensagem="Esta ação para o serviço, remove arquivos service.conf e reinicia o AnyDesk para gerar novo ID."
          textoConfirmar="Resetar"
          textoCancelar="Cancelar"
          onConfirmar={() => void run()}
          onCancelar={() => setConfirming(false)}
        />
      )}
    </FeatureContainer>
  );
}

export default ResetAnyDesk;
