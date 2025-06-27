package main

import (
	"BG-SupTec/internal/syscmd"
	"BG-SupTec/internal/sysinfo"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type App struct {
	ctx           context.Context
	senhaHasheada string
}

// Carrega o .env ao iniciar o app
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

		a.emitLogAtivacao("windows", "--> Definindo servidor KMS: 107.173.230.24...")
		output, _ = syscmd.RunCommand("", "cscript", slmgrPath, "/skms", "107.173.230.24")
		a.emitLogAtivacao("windows", output)

		a.emitLogAtivacao("windows", "--> Tentando ativar...")
		output, err = syscmd.RunCommand("", "cscript", slmgrPath, "/ato")
		a.emitLogAtivacao("windows", output)

		if err != nil {
			a.emitLogAtivacao("windows", "--> Falhou. Tentando servidor de fallback: kms9.msguides.com...")
			syscmd.RunCommand("", "cscript", slmgrPath, "/skms", "kms9.msguides.com")
			a.emitLogAtivacao("windows", "--> Tentando ativar novamente...")
			output, err = syscmd.RunCommand("", "cscript", slmgrPath, "/ato")
			a.emitLogAtivacao("windows", output)
		}

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

		a.emitLogAtivacao("office", "--> Fechando processos do Office (Word, Excel, etc.)...")
		syscmd.RunCommand("", "taskkill", "/f", "/im", "winword.exe")
		syscmd.RunCommand("", "taskkill", "/f", "/im", "excel.exe")
		syscmd.RunCommand("", "taskkill", "/f", "/im", "powerpnt.exe")
		syscmd.RunCommand("", "taskkill", "/f", "/im", "outlook.exe")

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

		a.emitLogAtivacao("office", "--> Definindo servidor KMS: 107.173.230.24...")
		output, _ = syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/sethst:107.173.230.24")
		a.emitLogAtivacao("office", output)

		a.emitLogAtivacao("office", "--> Tentando ativar...")
		output, err = syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/act")
		a.emitLogAtivacao("office", output)

		if err != nil {
			a.emitLogAtivacao("office", "--> Falhou. Tentando servidor de fallback: kms9.msguides.com...")
			syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/sethst:kms9.msguides.com")
			a.emitLogAtivacao("office", "--> Tentando ativar novamente...")
			output, err = syscmd.RunCommand(officePath, "cscript", "ospp.vbs", "/act")
			a.emitLogAtivacao("office", output)
		}

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
	}

	for _, path := range possiblePaths {
		if _, err := os.Stat(filepath.Join(path, "ospp.vbs")); err == nil {
			return path, nil
		}
	}

	return "", errors.New("a pasta de instalação do Office (Office16) não foi encontrada")
}

func (a *App) ObterInformacoesSistema() (sysinfo.InfoSistema, error) {
	return sysinfo.GetInfo()
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
