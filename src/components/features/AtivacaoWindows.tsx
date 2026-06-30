import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface AtivacaoWindowsProps {
  onVoltar: () => void;
}

function AtivacaoWindows({ onVoltar }: AtivacaoWindowsProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);
  const [versaoSelecionada, setVersaoSelecionada] = useState("pro");

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  useLogEvent(EVENTOS.logAtivacaoWindows, adicionarLog);

  async function iniciar() {
    setEmExecucao(true);
    setLogLines(["Iniciando ativação do Windows..."]);

    let unlisten: (() => void) | undefined;
    const promessaDeConclusao = new Promise<void>((resolve) => {
      listen(EVENTOS.ativacaoWindowsFinalizado, () => resolve()).then((fn) => {
        unlisten = fn;
      });
    });

    try {
      await invoke("ativar_windows", { versao: versaoSelecionada });
      await promessaDeConclusao;
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      unlisten?.();
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Ativação do Windows">
      <div className="flex min-h-0 flex-grow flex-col">
        <p className="mb-6 flex-shrink-0 text-center opacity-90">
          Esta função tentará ativar sua versão do Windows usando servidores KMS públicos. Requer
          privilégios de Administrador.
        </p>

        <div className="mx-auto w-full max-w-xl flex-shrink-0">
          <div className="flex items-end gap-4 text-left">
            <div className="flex-grow">
              <label htmlFor="versao-windows" className="mb-2 block font-bold text-text-light">
                Selecione a versão:
              </label>
              <select
                id="versao-windows"
                value={versaoSelecionada}
                onChange={(e) => setVersaoSelecionada(e.target.value)}
                disabled={emExecucao}
                className="w-full rounded-md border border-structural-purple bg-dark-blue-light p-3 text-base text-text-light focus:outline-none focus:ring-1 focus:ring-structural-purple disabled:cursor-not-allowed disabled:opacity-60"
              >
                <option value="pro">Windows 10/11 Pro</option>
                <option value="home">Windows 10/11 Home</option>
                <option value="education">Windows 10/11 Education</option>
                <option value="enterprise">Windows 10/11 Enterprise</option>
              </select>
            </div>
            <button
              className="flex-shrink-0 cursor-pointer rounded-lg border-none bg-accent-orange px-6 py-3 text-lg font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
              onClick={iniciar}
              disabled={emExecucao}
            >
              {emExecucao ? "Ativando..." : "Ativar"}
            </button>
          </div>
        </div>

        <LogPanel logLines={logLines} />
      </div>

      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
    </FeatureContainer>
  );
}

export default AtivacaoWindows;
