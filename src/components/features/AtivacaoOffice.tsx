import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface AtivacaoOfficeProps {
  onVoltar: () => void;
}

function AtivacaoOffice({ onVoltar }: AtivacaoOfficeProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);
  const [versaoSelecionada, setVersaoSelecionada] = useState("2024");

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  useLogEvent(EVENTOS.logAtivacaoOffice, adicionarLog);

  async function iniciar() {
    setEmExecucao(true);
    setLogLines(["Iniciando ativação do Office..."]);

    let unlisten: (() => void) | undefined;
    const promessaDeConclusao = new Promise<void>((resolve) => {
      listen(EVENTOS.ativacaoOfficeFinalizado, () => resolve()).then((fn) => {
        unlisten = fn;
      });
    });

    try {
      await invoke("ativar_office", { versao: versaoSelecionada });
      await promessaDeConclusao;
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      unlisten?.();
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Ativação do Office">
      <div className="flex min-h-0 flex-grow flex-col">
        <p className="mb-6 flex-shrink-0 text-center opacity-90">
          Esta função tentará ativar sua versão do Office usando servidores KMS públicos. Requer
          privilégios de Administrador.
        </p>

        <div className="mx-auto w-full max-w-xl flex-shrink-0">
          <div className="flex items-end gap-4 text-left">
            <div className="flex-grow">
              <label htmlFor="versao-office" className="mb-2 block font-bold text-text-light">
                Selecione a versão:
              </label>
              <select
                id="versao-office"
                value={versaoSelecionada}
                onChange={(e) => setVersaoSelecionada(e.target.value)}
                disabled={emExecucao}
                className="w-full rounded-md border border-structural-purple bg-dark-blue-light p-3 text-base text-text-light focus:outline-none focus:ring-1 focus:ring-structural-purple disabled:cursor-not-allowed disabled:opacity-60"
              >
                <option value="2024">Office 2024</option>
                <option value="2021">Office 2021</option>
                <option value="2016">Office 2016</option>
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

export default AtivacaoOffice;
