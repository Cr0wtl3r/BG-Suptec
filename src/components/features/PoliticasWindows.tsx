import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface PoliticasWindowsProps {
  onVoltar: () => void;
}

type PendingAction = {
  titulo: string;
  mensagem: string;
  run: () => Promise<void>;
};

function PoliticasWindows({ onVoltar }: PoliticasWindowsProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [updateMode, setUpdateMode] = useState("notificationsOnly");
  const [defenderPolicy, setDefenderPolicy] = useState("enable");
  const [onedriveEnabled, setOnedriveEnabled] = useState(false);
  const [powerProfile, setPowerProfile] = useState("highPerformance");
  const [pending, setPending] = useState<PendingAction | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logPoliticas, addLog);

  async function execute(action: PendingAction) {
    setPending(null);
    setBusy(true);
    setLogs([]);
    try {
      await action.run();
      addLog("Operação concluída.");
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  function confirm(action: PendingAction) {
    setPending(action);
  }

  return (
    <FeatureContainer titulo="Políticas do Windows">
      <div className="grid min-h-0 flex-grow grid-cols-1 gap-4 overflow-y-auto pr-2 lg:grid-cols-2">
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Windows Update</h3>
          <select
            value={updateMode}
            onChange={(event) => setUpdateMode(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            <option value="disable">Desativar</option>
            <option value="notificationsOnly">Somente notificações</option>
            <option value="enable">Ativar</option>
          </select>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Aplicar política de Windows Update?",
                mensagem:
                  "Esta ação altera políticas e serviço do Windows Update. Use somente quando a manutenção exigir esse comportamento.",
                run: () => invoke("configurar_windows_update", { modo: updateMode }),
              })
            }
            className="w-full rounded-lg bg-accent-orange px-4 py-3 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Aplicar Windows Update
          </button>
        </section>

        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Defender e SmartScreen</h3>
          <select
            value={defenderPolicy}
            onChange={(event) => setDefenderPolicy(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            <option value="enable">Ativar Defender</option>
            <option value="disable">Desativar Defender</option>
          </select>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Alterar política do Defender?",
                mensagem:
                  "Esta ação altera políticas do Microsoft Defender e pode reduzir a proteção do computador.",
                run: () => invoke("configurar_defender", { politica: defenderPolicy }),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Aplicar Defender
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Desativar SmartScreen?",
                mensagem:
                  "Esta ação desativa políticas do SmartScreen. Confirme somente se isso fizer parte do atendimento.",
                run: () => invoke("desativar_smartscreen"),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Desativar SmartScreen
          </button>
        </section>

        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Integrações</h3>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={onedriveEnabled}
              onChange={(event) => setOnedriveEnabled(event.target.checked)}
              className="h-4 w-4 accent-accent-orange"
            />
            <span>OneDrive ativo</span>
          </label>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Alterar integração do OneDrive?",
                mensagem: "Esta ação grava política de integração do OneDrive no Windows.",
                run: () => invoke("configurar_onedrive", { ativar: onedriveEnabled }),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Aplicar OneDrive
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Aplicar políticas SMB do Windows 11?",
                mensagem:
                  "Esta ação permite convidado inseguro e desativa exigência de assinatura SMB para compatibilidade.",
                run: () => invoke("aplicar_politicas_windows11"),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Aplicar SMB Windows 11
          </button>
        </section>

        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Sistema</h3>
          <select
            value={powerProfile}
            onChange={(event) => setPowerProfile(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            <option value="balanced">Equilibrado</option>
            <option value="highPerformance">Alto desempenho</option>
          </select>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Aplicar perfil de energia?",
                mensagem: "Esta ação altera o plano de energia ativo do Windows.",
                run: () => invoke("aplicar_perfil_energia", { perfil: powerProfile }),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Aplicar Energia
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              confirm({
                titulo: "Restaurar menu clássico?",
                mensagem: "Esta ação altera o menu de contexto do Windows 11 no HKCU.",
                run: () => invoke("restaurar_menu_classico_windows11"),
              })
            }
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Menu Clássico Windows 11
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
          titulo={pending.titulo}
          mensagem={pending.mensagem}
          textoConfirmar="Confirmar"
          textoCancelar="Cancelar"
          onConfirmar={() => void execute(pending)}
          onCancelar={() => setPending(null)}
        />
      )}
    </FeatureContainer>
  );
}

export default PoliticasWindows;
