import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface DiscoWindowsProps {
  onVoltar: () => void;
}

interface WindowsEditionInfo {
  current: string;
  targets: string[];
}

function DiscoWindows({ onVoltar }: DiscoWindowsProps) {
  const [logs, setLogs] = useState<string[]>(["Aguardando ação..."]);
  const [busy, setBusy] = useState(false);
  const [info, setInfo] = useState<WindowsEditionInfo | null>(null);
  const [edicao, setEdicao] = useState("");
  const [chave, setChave] = useState("");
  const [pending, setPending] = useState<"mbr" | "edition" | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogs((current) => [...current, `[${timestamp}] ${message}`]);
  }

  useLogEvent(EVENTOS.logDisco, addLog);

  useEffect(() => {
    void carregarEdicoes();
  }, []);

  async function carregarEdicoes() {
    try {
      const result = await invoke<WindowsEditionInfo>("obter_edicoes_windows");
      setInfo(result);
      setEdicao(result.targets[0] ?? "");
    } catch (err) {
      addLog(`Não foi possível consultar edições: ${err}`);
    }
  }

  async function runMbr() {
    setPending(null);
    setBusy(true);
    setLogs([]);
    try {
      await invoke("converter_mbr_para_gpt");
      addLog("Conversão MBR2GPT concluída.");
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  async function runEdition() {
    setPending(null);
    setBusy(true);
    setLogs([]);
    try {
      const result = await invoke<string>("mudar_edicao_windows", { edicao, chave });
      addLog(result);
    } catch (err) {
      addLog(`ERRO: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <FeatureContainer titulo="Disco e Edição do Windows">
      <div className="grid flex-shrink-0 grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">MBR2GPT</h3>
          <button
            type="button"
            disabled={busy}
            onClick={() => setPending("mbr")}
            className="w-full rounded-lg bg-accent-orange px-4 py-3 font-bold text-dark-blue-bg disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Converter MBR para GPT
          </button>
        </section>
        <section className="space-y-3 rounded-lg bg-dark-blue-bg/30 p-4">
          <h3 className="m-0 text-lg font-semibold">Edição do Windows</h3>
          <p className="m-0 text-sm opacity-80">Atual: {info?.current ?? "Não consultada"}</p>
          <select
            value={edicao}
            onChange={(event) => setEdicao(event.target.value)}
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          >
            {info?.targets.map((target) => (
              <option key={target} value={target}>
                {target}
              </option>
            ))}
          </select>
          <input
            value={chave}
            onChange={(event) => setChave(event.target.value)}
            placeholder="XXXXX-XXXXX-XXXXX-XXXXX-XXXXX"
            className="w-full rounded-md border border-structural-purple bg-dark-blue-light px-3 py-2"
          />
          <button
            type="button"
            disabled={busy || !edicao || !chave}
            onClick={() => setPending("edition")}
            className="w-full rounded-lg bg-structural-purple px-4 py-3 font-semibold hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Mudar Edição
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
          titulo={pending === "mbr" ? "Converter disco para GPT?" : "Mudar edição do Windows?"}
          mensagem={
            pending === "mbr"
              ? "Esta ação valida e converte o disco do sistema com mbr2gpt. Faça somente em VM/sessão elevada adequada."
              : "Esta ação usa DISM /Set-Edition com a chave informada. Não executa ativação automática."
          }
          textoConfirmar="Confirmar"
          textoCancelar="Cancelar"
          onConfirmar={() => void (pending === "mbr" ? runMbr() : runEdition())}
          onCancelar={() => setPending(null)}
        />
      )}
    </FeatureContainer>
  );
}

export default DiscoWindows;
