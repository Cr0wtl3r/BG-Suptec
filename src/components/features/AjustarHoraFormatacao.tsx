import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface AjustarHoraFormatacaoProps {
  onVoltar: () => void;
}

function AjustarHoraFormatacao({ onVoltar }: AjustarHoraFormatacaoProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  useLogEvent(EVENTOS.logAjustarHoraFormatacao, adicionarLog);

  async function iniciar() {
    setEmExecucao(true);
    setLogLines([]);

    try {
      await invoke("ajustar_hora_formatacao");
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Ajustar Hora da Formatação">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Esta função configura o serviço de horário do Windows, sincroniza com servidores NTP e
            ajusta a data de instalação do sistema para a data/hora atual. Requer privilégios de
            Administrador.
          </p>
          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={iniciar}
            disabled={emExecucao}
          >
            {emExecucao ? "Ajustando..." : "Ajustar Hora"}
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

export default AjustarHoraFormatacao;
