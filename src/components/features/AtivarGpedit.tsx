import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface AtivarGpeditProps {
  onVoltar: () => void;
}

function AtivarGpedit({ onVoltar }: AtivarGpeditProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  useLogEvent(EVENTOS.logAtivarGpedit, adicionarLog);

  async function iniciar() {
    setEmExecucao(true);
    setLogLines([]);

    try {
      const instalados = await invoke<number>("ativar_gpedit");
      adicionarLog(`${instalados} pacote(s) instalado(s) com sucesso.`);
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Ativar Editor de Política de Grupo (gpedit.msc)">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Esta função instala o Editor de Política de Grupo (gpedit.msc) em edições do Windows
            (como a Home) que não o possuem por padrão. Requer privilégios de Administrador e pode
            demorar alguns minutos.
          </p>
          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={iniciar}
            disabled={emExecucao}
          >
            {emExecucao ? "Ativando..." : "Ativar o Gpedit.msc"}
          </button>
        </div>

        <LogPanel logLines={logLines} />
      </div>

      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
    </FeatureContainer>
  );
}

export default AtivarGpedit;
