package main

import (
	"BG-SupTec/internal/syscmd"
	"BG-SupTec/internal/sysinfo"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type App struct {
	ctx           context.Context
	senhaHasheada string
}

func init() {
	err := godotenv.Load()
	if err != nil {
		log.Println("Aviso: .env não carregado ou não encontrado. Variáveis de ambiente serão usadas.")
	}
}

func NewApp() *App {
	senha := os.Getenv("PASSWORD")
	if senha == "" {
		log.Println("ERRO: PASSWORD não definido no .env ou variáveis de ambiente.")
	}
	return &App{senhaHasheada: senha}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Login(senha string) bool {
	hasher := sha256.New()
	hasher.Write([]byte(senha))
	hashSenha := hex.EncodeToString(hasher.Sum(nil))
	return hashSenha == a.senhaHasheada
}

func (a *App) emitLogAtivacao(tipo string, mensagem string) {
	eventName := "log:ativacao:" + tipo
	runtime.EventsEmit(a.ctx, eventName, mensagem)
}

func (a *App) ExecutarComando(comando string, args []string) (string, error) {
	return syscmd.RunCommand("", comando, args...)
}

func (a *App) VerificarIPDisponivel(ip string) (bool, error) {
	if net.ParseIP(ip) == nil {
		return false, errors.New("formato de IP inválido")
	}
	cmd := exec.Command("ping", "-n", "1", "-w", "1000", ip)
	err := cmd.Run()
	return err != nil, nil
}

func (a *App) AlterarIP(interfaceName, novoIP, mascara, gateway string) error {
	disponivel, err := a.VerificarIPDisponivel(novoIP)
	if err != nil {
		return fmt.Errorf("erro ao verificar disponibilidade do IP: %v", err)
	}
	if !disponivel {
		return errors.New("o IP informado já está em uso na rede")
	}
	if interfaceName == "" {
		return errors.New("não foi possível identificar a interface de rede para alteração")
	}

	cmd := fmt.Sprintf(`netsh interface ip set address name="%s" static %s %s %s`,
		interfaceName, novoIP, mascara, gateway)

	output, err := syscmd.RunCommand("", "cmd", "/c", cmd)
	if err != nil {
		return fmt.Errorf("erro ao alterar IP: %v - %s", err, output)
	}
	return nil
}

func (a *App) AlterarNomeComputador(novoNome string) error {
	if len(novoNome) == 0 || len(novoNome) > 15 {
		return errors.New("nome do computador deve ter entre 1 e 15 caracteres")
	}
	novoNome = strings.ReplaceAll(novoNome, " ", "")

	cmd := fmt.Sprintf("Rename-Computer -NewName %s -Force -PassThru", novoNome)
	output, err := syscmd.RunCommand("", "powershell", "-NoProfile", "-Command", cmd)
	if err != nil {
		return fmt.Errorf("erro ao alterar nome do computador: %v - %s", err, output)
	}
	return nil
}

func (a *App) AlterarDNS(interfaceName, dnsPrimario, dnsSecundario string) error {
	if net.ParseIP(dnsPrimario) == nil {
		return errors.New("DNS primário tem formato inválido")
	}
	if interfaceName == "" {
		return errors.New("não foi possível identificar a interface de rede para alteração")
	}

	cmdPrimario := fmt.Sprintf(`netsh interface ip set dns name="%s" static %s`, interfaceName, dnsPrimario)
	output, err := syscmd.RunCommand("", "cmd", "/c", cmdPrimario)
	if err != nil {
		return fmt.Errorf("erro ao configurar DNS primário: %v - %s", err, output)
	}

	if dnsSecundario != "" && net.ParseIP(dnsSecundario) != nil {
		cmdSecundario := fmt.Sprintf(`netsh interface ip add dns name="%s" %s index=2`, interfaceName, dnsSecundario)
		output, err = syscmd.RunCommand("", "cmd", "/c", cmdSecundario)
		if err != nil {
			return fmt.Errorf("erro ao configurar DNS secundário: %v - %s", err, output)
		}
	}
	return nil
}

func (a *App) ObterInformacoesSistema() (sysinfo.InfoSistema, error) {
	return sysinfo.GetInfo()
}


func (a *App) AtivarWindows(versao string) {
	go func() {
		time.Sleep(100 * time.Millisecond)
		keys := map[string]string{
			"pro":        "W269N-WFGWX-YVC9B-4J6C9-T83GX",
			"education":  "NW6C2-QMPVW-D7KKK-3GKT6-VCFB2",
			"enterprise": "NPPR9-FWDCX-D2C8J-H872K-2YT43",
			"home":       "TX9XD-98N7V-6WMQ6-BX7FG-H8Q99",
		}
		key, ok := keys[versao]
		if !ok {
			a.emitLogAtivacao("windows", "ERRO: Versão do Windows inválida.")
			return
		}
		a.emitLogAtivacao("windows", "Iniciando ativação para Windows "+versao+"...")
		a.emitLogAtivacao("windows", "--> Instalando chave do produto (GVLK)...")
		slmgrPath := filepath.Join(os.Getenv("SystemRoot"), "System32", "slmgr.vbs")
		output, err := syscmd.RunCommand("", "cscript", slmgrPath, "/ipk", key)
		a.emitLogAtivacao("windows", output)
		if err != nil {
			a.emitLogAtivacao("windows", "--- FALHA AO INSTALAR CHAVE ---")
			return
		}
		a.emitLogAtivacao("windows", "--> Definindo servidor KMS: kms.msguides.com...")
		output, _ = syscmd.RunCommand("", "cscript", slmgrPath, "/skms", "kms.msguides.com")
		a.emitLogAtivacao("windows", output)
		a.emitLogAtivacao("windows", "--> Tentando ativar...")
		output, err = syscmd.RunCommand("", "cscript", slmgrPath, "/ato")
		a.emitLogAtivacao("windows", output)
		if err == nil {
			a.emitLogAtivacao("windows", "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---")
		} else {
			a.emitLogAtivacao("windows", "--- FALHA NA ATIVAÇÃO ---")
		}
	}()
}

func (a *App) AtivarOffice(versao string) {
	go func() {
		a.emitLogAtivacao("office", "Iniciando ativação para Office...")
		officePath, err := findOfficePath()
		if err != nil {
			a.emitLogAtivacao("office", "ERRO: "+err.Error())
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO ---")
			return
		}
		a.emitLogAtivacao("office", "Pasta do Office encontrada em: "+officePath)
		a.emitLogAtivacao("office", "--> Fechando processos do Office...")
		syscmd.RunCommand("", "taskkill", "/f", "/im", "winword.exe", "/im", "excel.exe", "/im", "powerpnt.exe", "/im", "outlook.exe")
		keys := map[string]string{
			"2016": "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99",
			"2021": "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
			"2024": "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
		}
		key, ok := keys[versao]
		if !ok {
			a.emitLogAtivacao("office", "ERRO: Versão do Office inválida.")
			return
		}
		a.emitLogAtivacao("office", "--> Instalando chave do produto...")
		output, err := syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/inpkey:"+key)
		a.emitLogAtivacao("office", output)
		if err != nil {
			a.emitLogAtivacao("office", "--- FALHA AO INSTALAR CHAVE ---")
			return
		}
		a.emitLogAtivacao("office", "--> Definindo servidor KMS: kms.msguides.com...")
		output, _ = syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/sethst:kms.msguides.com")
		a.emitLogAtivacao("office", output)
		a.emitLogAtivacao("office", "--> Tentando ativar...")
		output, err = syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/act")
		a.emitLogAtivacao("office", output)
		if err == nil {
			a.emitLogAtivacao("office", "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---")
		} else {
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO ---")
		}
	}()
}

func findOfficePath() (string, error) {
	programFiles := os.Getenv("ProgramFiles")
	programFilesX86 := os.Getenv("ProgramFiles(x86)")
	possiblePaths := []string{
		filepath.Join(programFiles, "Microsoft Office", "Office16"),
		filepath.Join(programFilesX86, "Microsoft Office", "Office16"),
		filepath.Join(programFiles, "Microsoft Office", "Office15"),
		filepath.Join(programFilesX86, "Microsoft Office", "Office15"),
	}
	for _, path := range possiblePaths {
		if _, err := os.Stat(filepath.Join(path, "ospp.vbs")); err == nil {
			return path, nil
		}
	}
	return "", errors.New("a pasta de instalação do Office não foi encontrada")
}

func (a *App) ExecutarComandoSimples(titulo string, comando string, args ...string) {
	go func() {
		eventName := "log:runner"
		a.emitLogRunner(eventName, "Iniciando tarefa: "+titulo)
		time.Sleep(100 * time.Millisecond)
		output, err := syscmd.RunCommand("", comando, args...)
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, "--- TAREFA CONCLUÍDA COM ERRO ---")
		} else {
			a.emitLogRunner(eventName, "--- TAREFA CONCLUÍDA COM SUCESSO ---")
		}
	}()
}

func (a *App) emitLogRunner(eventName string, mensagem string) {
	runtime.EventsEmit(a.ctx, eventName, mensagem)
}