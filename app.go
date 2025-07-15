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
	"regexp"
	"strings"
	"time"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

var compiledPasswordHash string

type App struct {
	ctx           context.Context
	senhaHasheada string
}

func init() {
	if compiledPasswordHash == "" {
		err := godotenv.Load()
		if err != nil {
			log.Println("Aviso: .env não carregado ou não encontrado. Tentando usar variáveis de ambiente ou senha compilada.")
		}
	}
}

func NewApp() *App {
	if compiledPasswordHash != "" {
		return &App{senhaHasheada: compiledPasswordHash}
	}

	hashDoEnv := os.Getenv("PASSWORD")
	if hashDoEnv == "" {
		log.Println("ERRO: PASSWORD (hash) não definido no .env ou como variável de ambiente. O login falhará.")
	}

	return &App{senhaHasheada: hashDoEnv}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Login(senha string) bool {
	hasher := sha256.New()
	hasher.Write([]byte(senha))
	hashSenhaDigitada := hex.EncodeToString(hasher.Sum(nil))
	return hashSenhaDigitada == a.senhaHasheada
}

func (a *App) emitLogAtivacao(tipo string, mensagem string) {
	eventName := "log:ativacao:" + tipo
	runtime.EventsEmit(a.ctx, eventName, mensagem)
}

func (a *App) ExecutarComando(comando string, args []string) (string, error) {
	output, err := syscmd.RunCommand("", comando, args...)

	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			isShutdownCancel := comando == "shutdown"
			for _, arg := range args {
				if arg == "/a" {
					isShutdownCancel = true
					break
				}
			}

			if isShutdownCancel && exitErr.ExitCode() == 1116 {
				return output, nil
			}
		}
	}
	return output, err
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
		output, err = syscmd.RunCommand("", "cscript", slmgrPath, "/skms", "kms.msguides.com")
		a.emitLogAtivacao("windows", output)
		if err != nil { a.emitLogAtivacao("windows", "Aviso: Falha ao definir KMS. Mas tentando ativar mesmo assim: " + err.Error()) }


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

type OfficeVersionInfo struct {
	ProdKey         string
	UnPKeys         []string
	LicensePatterns []string
	KMS_Servers     []string
}

func (a *App) AtivarOffice(versao string) {
	go func() {
		a.emitLogAtivacao("office", "Iniciando ativação para Office...")
		officePath, err := findOfficePathGo()
		if err != nil {
			a.emitLogAtivacao("office", "ERRO: "+err.Error())
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO GERAL ---")
			return
		}
		a.emitLogAtivacao("office", "Pasta do Office encontrada em: "+officePath)

		versions := map[string]OfficeVersionInfo{
			"2016": {
				ProdKey:         "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99",
				UnPKeys:         []string{"BTDRB", "KHGM9", "CPQVG"},
				LicensePatterns: []string{`proplusvl_kms.*\.xrm-ms`},
				KMS_Servers:     []string{"107.173.230.24","kms9.msguides.com", "23.226.136.46", "kms8.msguides.com"},
			},
			"2021": {
				ProdKey:         "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
				UnPKeys:         []string{"6F7TH"},
				LicensePatterns: []string{`ProPlus2021VL_KMS.*\.xrm-ms`},
				KMS_Servers:     []string{"107.173.230.24","kms9.msguides.com", "23.226.136.46", "kms8.msguides.com"},
			},
			"2024": {
				ProdKey:         "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
				UnPKeys:         []string{"6F7TH"},
				LicensePatterns: []string{`ProPlus2024VL_KMS.*\.xrm-ms`},
				KMS_Servers:     []string{"107.173.230.24","kms9.msguides.com", "23.226.136.46", "kms8.msguides.com"},
			},
		}

		info, ok := versions[versao]
		if !ok {
			a.emitLogAtivacao("office", "ERRO: Versão do Office inválida. As versões suportadas são 2016, 2021, 2024.")
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO ---")
			return
		}

		a.emitLogAtivacao("office", "--> Fechando processos do Office...")
		_, _ = syscmd.RunCommand("", "taskkill", "/f", "/im", "winword.exe", "/im", "excel.exe", "/im", "powerpnt.exe", "/im", "outlook.exe")
		time.Sleep(1 * time.Second)

		osppPath := filepath.Join(officePath, "ospp.vbs")

		for _, unpkey := range info.UnPKeys {
			a.emitLogAtivacao("office", fmt.Sprintf("--> Desinstalando chave do produto existente (%s)...", unpkey))
			output, err := syscmd.RunCommand(officePath, "cscript", osppPath, "/unpkey:"+unpkey)
			if err != nil {
				a.emitLogAtivacao("office", fmt.Sprintf("Aviso: Falha ao desinstalar chave %s (pode não existir): %v - %s", unpkey, err, output))
			} else {
				a.emitLogAtivacao("office", output)
			}
		}

		if len(info.LicensePatterns) > 0 {
			licensesDirCandidates := []string{
				filepath.Join(officePath, "..", "root", "Licenses16"),
				filepath.Join(officePath, "..", "root", "Licenses15"),
			}
			foundLicensesDir := ""
			for _, dir := range licensesDirCandidates {
				if _, err := os.Stat(dir); err == nil {
					foundLicensesDir = dir
					break
				}
			}

			if foundLicensesDir == "" {
				a.emitLogAtivacao("office", "Aviso: Diretório de licenças KMS não encontrado. Pulando instalação de licenças.")
			} else {
				files, err := os.ReadDir(foundLicensesDir)
				if err != nil {
					a.emitLogAtivacao("office", fmt.Sprintf("Erro ao ler diretório de licenças %s: %v", foundLicensesDir, err))
				} else {
					for _, pattern := range info.LicensePatterns {
						re := regexp.MustCompile(pattern)
						for _, file := range files {
							if !file.IsDir() && re.MatchString(file.Name()) {
								licensePath := filepath.Join(foundLicensesDir, file.Name())
								a.emitLogAtivacao("office", fmt.Sprintf("--> Instalando licença KMS: %s", file.Name()))
								output, err := syscmd.RunCommand(officePath, "cscript", osppPath, "/inslic:"+licensePath)
								a.emitLogAtivacao("office", output)
								if err != nil {
									a.emitLogAtivacao("office", fmt.Sprintf("--- FALHA AO INSTALAR LICENÇA %s ---", file.Name()))
								}
							}
						}
					}
				}
			}
		}

		a.emitLogAtivacao("office", fmt.Sprintf("--> Instalando chave do produto GVLK (%s)...", info.ProdKey))
		output, err := syscmd.RunCommand(officePath, "cscript", osppPath, "/inpkey:"+info.ProdKey)
		a.emitLogAtivacao("office", output)
		if err != nil {
			a.emitLogAtivacao("office", "--- FALHA AO INSTALAR CHAVE DO PRODUTO GVLK ---")
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO ---")
			return
		}

		a.emitLogAtivacao("office", "--> Definindo porta KMS: 1688 (padrão)...")
		output, err = syscmd.RunCommand(officePath, "cscript", osppPath, "/setprt:1688")
		a.emitLogAtivacao("office", output)
		if err != nil { a.emitLogAtivacao("office", "Aviso: Falha ao definir porta KMS. " + err.Error()) }


		activationSuccessful := false
		for _, server := range info.KMS_Servers {
			a.emitLogAtivacao("office", fmt.Sprintf("--> Definindo servidor KMS: %s...", server))
			output, err = syscmd.RunCommand(officePath, "cscript", osppPath, "/sethst:"+server)
			a.emitLogAtivacao("office", output)
			if err != nil {
				a.emitLogAtivacao("office", fmt.Sprintf("Aviso: Falha ao definir servidor %s: %v", server, err))
				continue
			}

			a.emitLogAtivacao("office", fmt.Sprintf("--> Tentando ativar com %s...", server))
			output, err = syscmd.RunCommand(officePath, "cscript", osppPath, "/act")
			a.emitLogAtivacao("office", output)

			if err == nil && (strings.Contains(output, "Product activation successful") || strings.Contains(output, "activated successfully") || strings.Contains(output, "licença ativa")) {
				a.emitLogAtivacao("office", "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---")
				activationSuccessful = true
				break
			} else {
				a.emitLogAtivacao("office", fmt.Sprintf("--- FALHA NA ATIVAÇÃO COM %s (tentando o próximo, se houver) ---", server))
				if err != nil {
					a.emitLogAtivacao("office", fmt.Sprintf("Detalhes do erro: %v", err))
				}
			}
		}

		if !activationSuccessful {
			a.emitLogAtivacao("office", "--- FALHA NA ATIVAÇÃO: NENHUM SERVIDOR KMS CONSEGUIU ATIVAR ---")
		}
	}()
}

func (a *App) AjustarHoraFormatacao() {
	go func() {
		eventName := "log:ajustar:hora:formatacao"
		a.emitLogRunner(eventName, "Iniciando ajuste da hora de formatação...")
		time.Sleep(100 * time.Millisecond) // Pequena pausa para UX

		// 1. Configurar o serviço de horário do Windows para iniciar automaticamente
		a.emitLogRunner(eventName, "--> Configurando serviço de horário (w32time) para iniciar automaticamente...")
		output, err := syscmd.RunCommand("", "sc", "config", "w32time", "start=auto")
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, fmt.Sprintf("ERRO: Falha ao configurar w32time: %v", err))
			a.emitLogRunner(eventName, "--- OPERAÇÃO FINALIZADA COM ERRO ---")
			return
		}

		// 2. Sincronizar a hora do computador com o servidor NTP
		a.emitLogRunner(eventName, "--> Sincronizando hora com servidor NTP (pool.ntp.br)...")
		// Usamos /resync para forçar a sincronização imediata
		output, err = syscmd.RunCommand("", "w32tm", "/config", "/syncfromflags:manual", "/manualpeerlist:\"pool.ntp.br\"", "/reliable:YES", "/update")
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, fmt.Sprintf("ERRO: Falha ao configurar servidor NTP: %v", err))
			a.emitLogRunner(eventName, "--- OPERAÇÃO FINALIZADA COM ERRO ---")
			return
		}

		// 3. Reiniciar o serviço de horário do Windows
		a.emitLogRunner(eventName, "--> Reiniciando o serviço de horário do Windows...")
		output, err = syscmd.RunCommand("", "net", "stop", "w32time")
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, fmt.Sprintf("Aviso: Falha ao parar w32time (pode já estar parado ou erro menor): %v", err))
		}
		time.Sleep(2 * time.Second) // Pequena pausa para garantir que o serviço parou

		output, err = syscmd.RunCommand("", "net", "start", "w32time")
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, fmt.Sprintf("ERRO: Falha ao iniciar w32time: %v", err))
			a.emitLogRunner(eventName, "--- OPERAÇÃO FINALIZADA COM ERRO ---")
			return
		}
		a.emitLogRunner(eventName, "Serviço de horário configurado e reiniciado com sucesso.")
		time.Sleep(1 * time.Second) // Pequena pausa para garantir a sincronização inicial

		// 4. Ajustar a data de formatação no registro
		// Obter o timestamp Unix atual
		now := time.Now().Unix()
		installDate := fmt.Sprintf("%d", now)

		a.emitLogRunner(eventName, fmt.Sprintf("--> Ajustando InstallDate no registro para o timestamp atual (%s)...", installDate))
		output, err = syscmd.RunCommand("", "reg", "add", `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`, "/v", "InstallDate", "/t", "REG_DWORD", "/d", installDate, "/f")
		a.emitLogRunner(eventName, output)
		if err != nil {
			a.emitLogRunner(eventName, fmt.Sprintf("ERRO: Falha ao ajustar InstallDate no registro: %v", err))
			a.emitLogRunner(eventName, "--- OPERAÇÃO FINALIZADA COM ERRO ---")
			return
		}

		a.emitLogRunner(eventName, "Hora de formatação ajustada com sucesso!")
		a.emitLogRunner(eventName, "--- OPERAÇÃO CONCLUÍDA ---")
	}()
}

func findOfficePathGo() (string, error) {
	programFiles := os.Getenv("ProgramFiles")
	programFilesX86 := os.Getenv("ProgramFiles(x86)")

	basePaths := []string{
		filepath.Join(programFiles, "Microsoft Office"),
		filepath.Join(programFilesX86, "Microsoft Office"),
	}

	versionFolders := []string{"Office16", "Office15"}

	for _, basePath := range basePaths {
		for _, versionFolder := range versionFolders {
			fullPath := filepath.Join(basePath, versionFolder)
			if _, err := os.Stat(filepath.Join(fullPath, "ospp.vbs")); err == nil {
				return fullPath, nil
			}
		}
	}
	return "", errors.New("a pasta de instalação do Office (com ospp.vbs) não foi encontrada")
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