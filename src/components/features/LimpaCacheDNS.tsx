import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";

interface LimpaCacheDNSProps {
  onVoltar: () => void;
}

function LimpaCacheDNS({ onVoltar }: LimpaCacheDNSProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  async function iniciar() {
    setEmExecucao(true);
    setLogLines(["Iniciando limpeza do cache DNS..."]);

    try {
      adicionarLog("Executando comando: ipconfig /flushdns");
      const resultado = await invoke<string>("limpar_cache_dns");
      adicionarLog("Resultado: " + resultado);
      adicionarLog("Cache DNS limpo com sucesso!");
      adicionarLog("--- Operação concluída ---");
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      adicionarLog("--- Operação finalizada com erro ---");
    } finally {
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Limpar Cache DNS">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Executa o comando 'ipconfig /flushdns' para limpar o cache de resolução de DNS local.
          </p>
          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={iniciar}
            disabled={emExecucao}
          >
            {emExecucao ? "Executando..." : "Executar Limpeza"}
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

export default LimpaCacheDNS;
