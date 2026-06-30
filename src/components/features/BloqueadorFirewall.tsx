import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import BotaoVoltar from "../shared/BotaoVoltar";

interface ProgramaInfo {
  nome: string;
  caminho: string;
}

interface ExecutavelInfo {
  caminho: string;
  bloqueado: boolean;
}

interface BloqueadorFirewallProps {
  onVoltar: () => void;
}

function BloqueadorFirewall({ onVoltar }: BloqueadorFirewallProps) {
  const [programasInstalados, setProgramasInstalados] = useState<ProgramaInfo[]>([]);
  const [executaveisParaAcao, setExecutaveisParaAcao] = useState<ExecutavelInfo[]>([]);
  const [processando, setProcessando] = useState(false);
  const [erro, setErro] = useState("");
  const [mensagemSucesso, setMensagemSucesso] = useState("");

  useEffect(() => {
    let cancelado = false;
    async function carregarProgramasInstalados() {
      setProcessando(true);
      try {
        const programas = await invoke<ProgramaInfo[]>("obter_programas_instalados");
        if (!cancelado) setProgramasInstalados(programas);
      } catch (e) {
        if (!cancelado) setErro(`Erro ao carregar programas instalados: ${e}`);
      } finally {
        if (!cancelado) setProcessando(false);
      }
    }
    carregarProgramasInstalados();
    return () => {
      cancelado = true;
    };
  }, []);

  async function adicionarExecutaveis(novosCaminhos: string[]) {
    setProcessando(true);
    setErro("");
    setMensagemSucesso("");
    try {
      const status = await invoke<Record<string, boolean>>("verificar_status_firewall", {
        caminhos: novosCaminhos,
      });
      setExecutaveisParaAcao((atual) => {
        const existentes = new Set(atual.map((e) => e.caminho));
        const adicionados = novosCaminhos
          .filter((caminho) => !existentes.has(caminho))
          .map((caminho) => ({ caminho, bloqueado: status[caminho] ?? false }));
        return [...atual, ...adicionados];
      });
    } catch (e) {
      setErro(`Erro ao verificar status no firewall: ${e}`);
    } finally {
      setProcessando(false);
    }
  }

  async function selecionarArquivo() {
    try {
      const caminho = await invoke<string | null>("selecionar_arquivo_exe");
      if (caminho) await adicionarExecutaveis([caminho]);
    } catch (e) {
      setErro(`Erro ao selecionar arquivo: ${e}`);
    }
  }

  async function selecionarProgramaInstalado(event: React.ChangeEvent<HTMLSelectElement>) {
    const caminhoPasta = event.target.value;
    if (!caminhoPasta) return;
    setProcessando(true);
    try {
      const executaveis = await invoke<string[]>("listar_executaveis", { caminho: caminhoPasta });
      if (executaveis.length > 0) {
        await adicionarExecutaveis(executaveis);
      } else {
        setErro("Nenhum arquivo .exe encontrado na pasta deste programa.");
      }
    } catch (e) {
      setErro(`Erro ao listar executáveis: ${e}`);
    } finally {
      setProcessando(false);
      event.target.value = "";
    }
  }

  function removerExecutavel(caminho: string) {
    setExecutaveisParaAcao((atual) => atual.filter((e) => e.caminho !== caminho));
  }

  async function executarAcao(acao: "bloquear" | "desbloquear") {
    const caminhos = executaveisParaAcao.map((e) => e.caminho);
    if (caminhos.length === 0) {
      setErro("Nenhum programa na lista para aplicar a ação.");
      return;
    }
    setProcessando(true);
    setErro("");
    setMensagemSucesso("");
    try {
      if (acao === "bloquear") {
        await invoke("bloquear_programas_firewall", { caminhos });
        setMensagemSucesso("Programas bloqueados com sucesso!");
      } else {
        await invoke("desbloquear_programas_firewall", { caminhos });
        setMensagemSucesso("Programas desbloqueados com sucesso!");
      }
      const statusAtualizado = await invoke<Record<string, boolean>>("verificar_status_firewall", {
        caminhos,
      });
      setExecutaveisParaAcao((atual) =>
        atual.map((exe) => ({ ...exe, bloqueado: statusAtualizado[exe.caminho] ?? exe.bloqueado })),
      );
    } catch (e) {
      setErro(`Erro ao ${acao} programas: ${e}`);
    } finally {
      setProcessando(false);
    }
  }

  return (
    <FeatureContainer titulo="Bloqueador de Programas no Firewall">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="flex-shrink-0 space-y-4">
          <p className="text-center opacity-90">
            Adicione programas à lista abaixo para bloquear ou desbloquear seu acesso à internet
            (entrada e saída).
          </p>
          <div className="grid grid-cols-1 items-center gap-4 rounded-lg bg-dark-blue-bg/30 p-4 md:grid-cols-2">
            <button
              type="button"
              onClick={selecionarArquivo}
              disabled={processando}
              className="w-full rounded-lg bg-structural-purple px-4 py-3 text-center transition-colors hover:bg-structural-purple-dim disabled:cursor-not-allowed disabled:opacity-60"
            >
              1. Selecionar Arquivo (.exe)...
            </button>
            <select
              onChange={selecionarProgramaInstalado}
              disabled={processando}
              defaultValue=""
              className="w-full appearance-none rounded-md border border-structural-purple bg-dark-blue-light px-4 py-3 disabled:cursor-not-allowed disabled:opacity-60"
            >
              <option value="" disabled>
                2. Ou escolha um programa instalado...
              </option>
              {programasInstalados.map((prog) => (
                <option key={prog.caminho} value={prog.caminho}>
                  {prog.nome}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="mt-4 min-h-0 flex-grow space-y-2 overflow-y-auto pr-2">
          {executaveisParaAcao.length > 0 ? (
            executaveisParaAcao.map((exe) => (
              <div
                key={exe.caminho}
                className="flex items-center gap-3 rounded-md bg-dark-blue-bg/50 p-2 text-sm"
              >
                <span
                  className={`h-3 w-3 flex-shrink-0 rounded-full ${
                    exe.bloqueado ? "bg-red-500" : "bg-green-500"
                  }`}
                  title={exe.bloqueado ? "Bloqueado" : "Não Bloqueado"}
                />
                <span className="flex-grow truncate" title={exe.caminho}>
                  {exe.caminho}
                </span>
                <button
                  type="button"
                  onClick={() => removerExecutavel(exe.caminho)}
                  className="flex-shrink-0 p-1 text-red-400 hover:text-red-200"
                  aria-label={`Remover ${exe.caminho} da lista`}
                >
                  &times;
                </button>
              </div>
            ))
          ) : (
            <div className="flex h-full items-center justify-center text-center italic opacity-60">
              Nenhum programa adicionado à lista.
            </div>
          )}
        </div>
        <div className="mt-2 h-6 text-center">
          {erro && <p className="text-red-500">{erro}</p>}
          {mensagemSucesso && <p className="text-green-400">{mensagemSucesso}</p>}
        </div>
        <div className="grid flex-shrink-0 grid-cols-2 gap-4 pt-2">
          <button
            type="button"
            onClick={() => executarAcao("bloquear")}
            disabled={processando || executaveisParaAcao.length === 0}
            className="w-full rounded-lg bg-red-600 p-4 text-lg font-bold hover:bg-red-700 disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Bloquear Selecionados
          </button>
          <button
            type="button"
            onClick={() => executarAcao("desbloquear")}
            disabled={processando || executaveisParaAcao.length === 0}
            className="w-full rounded-lg bg-green-600 p-4 text-lg font-bold hover:bg-green-700 disabled:cursor-not-allowed disabled:bg-gray-600"
          >
            Desbloquear Selecionados
          </button>
        </div>
      </div>
      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
    </FeatureContainer>
  );
}

export default BloqueadorFirewall;
