import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";

interface DesativaHibernacaoProps {
  onVoltar: () => void;
}

function DesativaHibernacao({ onVoltar }: DesativaHibernacaoProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  async function iniciar() {
    setEmExecucao(true);
    setLogLines(["Iniciando desativação da hibernação do Windows..."]);

    try {
      adicionarLog("Executando comando: powercfg /h off");
      const resultado = await invoke<string>("desativar_hibernacao");
      adicionarLog("Resultado: " + resultado);
      adicionarLog("Hibernação desativada com sucesso!");
      adicionarLog("O arquivo hiberfil.sys foi removido, liberando espaço em disco.");
      adicionarLog("--- Operação concluída ---");
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      adicionarLog("--- Operação finalizada com erro ---");
    } finally {
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Desativar Hibernação do Windows">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Desativa o recurso de hibernação e remove o arquivo hiberfil.sys, liberando espaço em
            disco.
          </p>
          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={iniciar}
            disabled={emExecucao}
          >
            {emExecucao ? "Executando..." : "Desativar Hibernação"}
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

export default DesativaHibernacao;
