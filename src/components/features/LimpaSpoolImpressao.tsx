import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";

interface LimpaSpoolImpressaoProps {
  onVoltar: () => void;
}

function LimpaSpoolImpressao({ onVoltar }: LimpaSpoolImpressaoProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  async function iniciar() {
    setEmExecucao(true);
    setLogLines(["Iniciando limpeza e reinício do Spool de Impressão..."]);

    try {
      adicionarLog("Parando o serviço Spooler, removendo arquivos travados e reiniciando...");
      const arquivosRemovidos = await invoke<number>("limpar_spool_impressao");
      adicionarLog(`${arquivosRemovidos} arquivo(s) de spool travado(s) removido(s).`);
      adicionarLog("Problemas de impressoras travadas devem estar resolvidos.");
      adicionarLog("--- Operação concluída ---");
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      adicionarLog("--- Operação finalizada com erro ---");
    } finally {
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Limpar e Reiniciar Spool de Impressão">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Para o serviço de Spooler de Impressão, remove arquivos de fila travados e reinicia o
            serviço — útil para resolver problemas de impressoras travadas.
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

export default LimpaSpoolImpressao;
