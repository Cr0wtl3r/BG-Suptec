import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface ReparosWindowsProps {
  onVoltar: () => void;
}

type ActionName = "corrigir_busca_menu_iniciar" | "executar_dism_restore_health" | "executar_sfc_scannow";

function ReparosWindows({ onVoltar }: ReparosWindowsProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<{ command: ActionName; title: string; message: string } | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logReparo, addLog);

  async function run(command: ActionName) {
    setPending(null);
    setBusy(true);
    setLogs([]);
    try {
      const result = await invoke<string | null>(command);
      if (result) addLog(result);
      addLog("Operação concluída.");
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Reparos do Windows">
      <div className="grid flex-shrink-0 grid-cols-1 gap-4 md:grid-cols-3">
        <button
          type="button"
          disabled={busy}
          onClick={() =>
            setPending({
              command: "corrigir_busca_menu_iniciar",
              title: "Corrigir busca/menu iniciar?",
              message: "Esta ação grava configuração no HKCU e reinicia o Explorer.",
            })
          }
          className="rounded-lg bg-accent-orange px-4 py-4 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
        >
          Corrigir Busca
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={() =>
            setPending({
              command: "executar_dism_restore_health",
              title: "Executar DISM?",
              message: "Esta ação executa DISM /Online /Cleanup-Image /RestoreHealth.",
            })
          }
          className="rounded-lg bg-structural-purple px-4 py-4 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
        >
          DISM RestoreHealth
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={() =>
            setPending({
              command: "executar_sfc_scannow",
              title: "Executar SFC?",
              message: "Esta ação executa SFC /scannow e pode levar vários minutos.",
            })
          }
          className="rounded-lg bg-structural-purple px-4 py-4 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
        >
          SFC /scannow
        </button>
      </div>
      <LogPanel logLines={logs} />
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
      {pending && (
        <Modal
          tipo="aviso"
          titulo={pending.title}
          mensagem={pending.message}
          textoConfirmar="Executar"
          textoCancelar="Cancelar"
          onConfirmar={() => void run(pending.command)}
          onCancelar={() => setPending(null)}
        />
      )}
    </FeatureContainer>
  );
}

export default ReparosWindows;
