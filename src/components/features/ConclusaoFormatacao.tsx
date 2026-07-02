import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface ConclusaoFormatacaoProps {
  onVoltar: () => void;
}

function ConclusaoFormatacao({ onVoltar }: ConclusaoFormatacaoProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [updateMode, setUpdateMode] = useState("notificationsOnly");
  const [restorePhotoViewer, setRestorePhotoViewer] = useState(true);
  const [disableOnedrive, setDisableOnedrive] = useState(true);
  const [disableHibernation, setDisableHibernation] = useState(true);
  const [powerProfile, setPowerProfile] = useState("desktop");
  const [restartAfter, setRestartAfter] = useState(false);
  const [confirming, setConfirming] = useState(false);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logFormatacao, addLog);

  async function run() {
    setConfirming(false);
    setBusy(true);
    setLogs([]);
    try {
      await invoke("executar_conclusao_formatacao", {
        options: {
          updateMode,
          restorePhotoViewer,
          disableOnedrive,
          disableHibernation,
          powerProfile,
          restartAfter,
        },
      });
      addLog("Workflow concluído.");
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Conclusão de Formatação">
      <div className="grid flex-shrink-0 grid-cols-1 gap-4 md:grid-cols-2">
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Políticas</h3>
          <select
            value={updateMode}
            onChange={(event) => setUpdateMode(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            <option value="disable">Desativar Windows Update</option>
            <option value="notificationsOnly">Somente notificações</option>
            <option value="enable">Ativar Windows Update</option>
          </select>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={restorePhotoViewer}
              onChange={(event) => setRestorePhotoViewer(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>Photo Viewer</span>
          </label>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={disableOnedrive}
              onChange={(event) => setDisableOnedrive(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>Desativar OneDrive</span>
          </label>
        </section>
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Finalização</h3>
          <select
            value={powerProfile}
            onChange={(event) => setPowerProfile(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            <option value="notebook">Notebook</option>
            <option value="desktop">Desktop</option>
          </select>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={disableHibernation}
              onChange={(event) => setDisableHibernation(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>Desativar hibernação</span>
          </label>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={restartAfter}
              onChange={(event) => setRestartAfter(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>Reiniciar ao final</span>
          </label>
        </section>
      </div>
      <button
        type="button"
        disabled={busy}
        onClick={() => setConfirming(true)}
        className="mt-4 flex-shrink-0 rounded-lg bg-accent-orange px-4 py-3 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
      >
        Executar Workflow
      </button>
      <LogPanel logLines={logs} />
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
      {confirming && (
        <Modal
          tipo="aviso"
          titulo="Executar conclusão de formatação?"
          mensagem="Esta ação orquestra ajustes de pós-formatação, registra auditoria e não remove a pasta dos scripts."
          textoConfirmar="Executar"
          textoCancelar="Cancelar"
          onConfirmar={() => void run()}
          onCancelar={() => setConfirming(false)}
        />
      )}
    </FeatureContainer>
  );
}

export default ConclusaoFormatacao;
