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
	"sort"
	"strings"
	"time"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

// --- Variáveis e Structs Globais ---

var compiledPasswordHash string

type TecladoInfo struct {
	ID        string `json:"id"`
	Nome      string `json:"nome"`
	TagIdioma string `json:"tagIdioma"`
}

var tecladosDisponiveis = []TecladoInfo{
	{ID: "0416:00000416", Nome: "Português (Brasil ABNT)", TagIdioma: "pt-BR"},
	{ID: "0416:00010416", Nome: "Português (Brasil ABNT2)", TagIdioma: "pt-BR"},
	{ID: "0816:00000816", Nome: "Português (Portugal)", TagIdioma: "pt-PT"},
	{ID: "0409:00000409", Nome: "Inglês (Estados Unidos)", TagIdioma: "en-US"},
	{ID: "0409:00020409", Nome: "Inglês (Estados Unidos-Internacional)", TagIdioma: "en-US"},
	{ID: "0c0a:0000040a", Nome: "Espanhol (Espanha - Internacional)", TagIdioma: "es-ES"},
	{ID: "080a:0000080a", Nome: "Espanhol (México/América Latina)", TagIdioma: "es-419"},
}

type App struct {
	ctx           context.Context
	senhaHasheada string
}

type OfficeVersionInfo struct {
	ProdKey         string
	UnPKeys         []string
	LicensePatterns []string
	KMS_Servers     []string
}

// --- Funções de Inicialização ---
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

// --- Métodos Principais da Aplicação ---
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Login(senha string) bool {
	hasher := sha256.New()
	hasher.Write([]byte(senha))
	hashSenhaDigitada := hex.EncodeToString(hasher.Sum(nil))
	return hashSenhaDigitada == a.senhaHasheada
}

func (a *App) ExecutarComando(comando string, args []string) (string, error) {
	output, err := syscmd.RunCommand("", comando, args...)
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			isShutdownCancel := comando == "shutdown" && len(args) > 0 && args[0] == "/a"
			if isShutdownCancel && exitErr.ExitCode() == 1116 {
				return "Nenhum desligamento agendado para cancelar.", nil
			}
		}
		return output, err
	}
	return output, nil
}

func (a *App) ReiniciarComputador() (string, error) {
	return syscmd.RunCommand("", "shutdown", "/r", "/t", "0")
}

// --- Funções de Sistema e Rede ---
func (a *App) ObterInformacoesSistema() (sysinfo.InfoSistema, error) {
	return sysinfo.GetInfo()
}

func (a *App) AlterarNomeComputador(novoNome string) error {
	if len(novoNome) == 0 || len(novoNome) > 15 {
		return errors.New("nome do computador deve ter entre 1 e 15 caracteres")
	}
	novoNome = strings.ReplaceAll(novoNome, " ", "")
	cmd := fmt.Sprintf("Rename-Computer -NewName %s -Force -PassThru", novoNome)
	_, err := syscmd.RunCommand("", "powershell", "-NoProfile", "-Command", cmd)
	return err
}

func (a *App) VerificarIPDisponivel(ip string) (bool, error) {
	if net.ParseIP(ip) == nil {
		return false, errors.New("formato de IP inválido")
	}
	cmd := exec.Command("ping", "-n", "1", "-w", "1000", ip)
	err := cmd.Run()
	return err != nil, nil
}

// MUDANÇA: Lógica para IP dinâmico adicionada
func (a *App) AlterarIP(interfaceName, novoIP, mascara, gateway string) error {
	if interfaceName == "" {
		return errors.New("não foi possível identificar a interface de rede para alteração")
	}

	// Se o campo de IP estiver vazio, configura para DHCP
	if novoIP == "" {
		cmd := fmt.Sprintf(`netsh interface ip set address name="%s" source=dhcp`, interfaceName)
		_, err := syscmd.RunCommand("", "cmd", "/c", cmd)
		return err
	}

	// Lógica original para IP estático
	disponivel, err := a.VerificarIPDisponivel(novoIP)
	if err != nil {
		return fmt.Errorf("erro ao verificar disponibilidade do IP: %v", err)
	}
	if !disponivel {
		return errors.New("o IP informado já está em uso na rede")
	}

	cmd := fmt.Sprintf(`netsh interface ip set address name="%s" static %s %s %s`, interfaceName, novoIP, mascara, gateway)
	_, err = syscmd.RunCommand("", "cmd", "/c", cmd)
	return err
}

// MUDANÇA: Lógica para DNS dinâmico adicionada
func (a *App) AlterarDNS(interfaceName, dnsPrimario, dnsSecundario string) error {
	if interfaceName == "" {
		return errors.New("não foi possível identificar a interface de rede para alteração")
	}

	// Se o campo DNS primário estiver vazio, configura para DHCP
	if dnsPrimario == "" {
		cmd := fmt.Sprintf(`netsh interface ip set dns name="%s" source=dhcp`, interfaceName)
		_, err := syscmd.RunCommand("", "cmd", "/c", cmd)
		return err
	}

	// Lógica original para DNS estático
	if net.ParseIP(dnsPrimario) == nil {
		return errors.New("DNS primário tem formato inválido")
	}

	cmdPrimario := fmt.Sprintf(`netsh interface ip set dns name="%s" static %s`, interfaceName, dnsPrimario)
	_, err := syscmd.RunCommand("", "cmd", "/c", cmdPrimario)
	if err != nil {
		return fmt.Errorf("erro ao configurar DNS primário: %v", err)
	}

	if dnsSecundario != "" {
		if net.ParseIP(dnsSecundario) == nil {
			return errors.New("DNS secundário tem formato inválido")
		}
		cmdSecundario := fmt.Sprintf(`netsh interface ip add dns name="%s" %s index=2`, interfaceName, dnsSecundario)
		_, err = syscmd.RunCommand("", "cmd", "/c", cmdSecundario)
		if err != nil {
			return fmt.Errorf("erro ao configurar DNS secundário: %v", err)
		}
	}
	
	return nil
}

// --- Funções de Layout de Teclado ---
func (a *App) ObterLayoutsDisponiveis() []TecladoInfo {
	sort.Slice(tecladosDisponiveis, func(i, j int) bool {
		return tecladosDisponiveis[i].Nome < tecladosDisponiveis[j].Nome
	})
	return tecladosDisponiveis
}

func (a *App) ObterLayoutAtivo() (string, error) {
	cmd := `(Get-WinUserLanguageList)[0].InputMethodTips[0]`
	output, err := syscmd.RunCommand("", "powershell", "-NoProfile", "-Command", cmd)
	if err != nil {
		return "", fmt.Errorf("falha ao obter layout ativo: %v", err)
	}
	return strings.TrimSpace(output), nil
}

func (a *App) AlterarLayoutDeTeclado(tagIdioma string) error {
	cmd := fmt.Sprintf(`Set-WinUserLanguageList -LanguageList %s -Force`, tagIdioma)
	_, err := syscmd.RunCommand("", "powershell", "-NoProfile", "-Command", cmd)
	return err
}

// --- Funções de Ativação e Correção ---
func (a *App) AtivarWindows(versao string) {
	go func() {
		eventName := "log:ativacao:windows"
		a.emitLogRunner(eventName, "Iniciando ativação para Windows "+versao+"...")
		keys := map[string]string{
			"pro":        "W269N-WFGWX-YVC9B-4J6C9-T83GX",
			"education":  "NW6C2-QMPVW-D7KKK-3GKT6-VCFB2",
			"enterprise": "NPPR9-FWDCX-D2C8J-H872K-2YT43",
			"home":       "TX9XD-98N7V-6WMQ6-BX7FG-H8Q99",
		}
		key, ok := keys[versao]
		if !ok {
			a.emitLogRunner(eventName, "ERRO: Versão do Windows inválida.")
			a.emitLogRunner(eventName, "--- FALHA NA ATIVAÇÃO ---")
			return
		}

		slmgrPath := filepath.Join(os.Getenv("SystemRoot"), "System32", "slmgr.vbs")

		a.runCommandAndLog(eventName, "Instalando chave do produto (GVLK)...", "cscript", slmgrPath, "/ipk", key)
		a.runCommandAndLog(eventName, "Definindo servidor KMS: kms.msguides.com...", "cscript", slmgrPath, "/skms", "kms.msguides.com")
		
		a.emitLogRunner(eventName, "--> Tentando ativar...")
		output, err := syscmd.RunCommand("", "cscript", slmgrPath, "/ato")
		a.emitLogRunner(eventName, output)
		if err == nil {
			a.emitLogRunner(eventName, "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---")
		} else {
			a.emitLogRunner(eventName, "--- FALHA NA ATIVAÇÃO ---")
		}
	}()
}

func (a *App) AtivarOffice(versao string) {
	go func() {
		eventName := "log:ativacao:office"
		a.emitLogRunner(eventName, "Iniciando ativação para Office...")
		officePath, err := findOfficePathGo()
		if err != nil {
			a.emitLogRunner(eventName, "ERRO: "+err.Error())
			a.emitLogRunner(eventName, "--- FALHA NA ATIVAÇÃO GERAL ---")
			return
		}
		a.emitLogRunner(eventName, "Pasta do Office encontrada em: "+officePath)

		versions := map[string]OfficeVersionInfo{
			"2016": {
				ProdKey:         "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99",
				UnPKeys:         []string{"BTDRB", "KHGM9", "CPQVG"},
				LicensePatterns: []string{`proplusvl_kms.*\.xrm-ms`},
				KMS_Servers:     []string{"kms8.msguides.com", "kms9.msguides.com"},
			},
			"2021": {
				ProdKey:         "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
				UnPKeys:         []string{"6F7TH"},
				LicensePatterns: []string{`ProPlus2021VL_KMS.*\.xrm-ms`},
				KMS_Servers:     []string{"kms8.msguides.com", "kms9.msguides.com"},
			},
			"2024": {
				ProdKey:         "FXYTK-NJJ8C-GB6DW-3DYQT-6F7TH",
				UnPKeys:         []string{"6F7TH"},
				LicensePatterns: []string{`ProPlus2024VL_KMS.*\.xrm-ms`},
				KMS_Servers:     []string{"kms8.msguides.com", "kms9.msguides.com"},
			},
		}

		info, ok := versions[versao]
		if !ok {
			a.emitLogRunner(eventName, "ERRO: Versão do Office inválida.")
			a.emitLogRunner(eventName, "--- FALHA NA ATIVAÇÃO ---")
			return
		}

		a.runCommandAndLog(eventName, "Fechando processos do Office...", "taskkill", "/f", "/im", "winword.exe", "/im", "excel.exe", "/im", "powerpnt.exe", "/im", "outlook.exe")
		time.Sleep(1 * time.Second)

		osppPath := filepath.Join(officePath, "ospp.vbs")

		for _, unpkey := range info.UnPKeys {
			a.runCommandAndLog(eventName, fmt.Sprintf("Desinstalando chave do produto existente (%s)...", unpkey), "cscript", osppPath, "/unpkey:"+unpkey)
		}

		if len(info.LicensePatterns) > 0 {
			a.instalarLicencasOffice(eventName, officePath, info.LicensePatterns)
		}

		a.runCommandAndLog(eventName, fmt.Sprintf("Instalando chave do produto GVLK (%s)...", info.ProdKey), "cscript", osppPath, "/inpkey:"+info.ProdKey)
		a.runCommandAndLog(eventName, "Definindo porta KMS: 1688 (padrão)...", "cscript", osppPath, "/setprt:1688")

		activationSuccessful := false
		for _, server := range info.KMS_Servers {
			a.runCommandAndLog(eventName, fmt.Sprintf("Definindo servidor KMS: %s...", server), "cscript", osppPath, "/sethst:"+server)
			
			a.emitLogRunner(eventName, fmt.Sprintf("--> Tentando ativar com %s...", server))
			output, err := syscmd.RunCommand(officePath, "cscript", osppPath, "/act")
			a.emitLogRunner(eventName, output)

			if err == nil && (strings.Contains(strings.ToLower(output), "product activation successful") || strings.Contains(strings.ToLower(output), "ativado com êxito")) {
				a.emitLogRunner(eventName, "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---")
				activationSuccessful = true
				break
			} else {
				a.emitLogRunner(eventName, fmt.Sprintf("--- Falha na ativação com %s. Tentando próximo... ---", server))
			}
		}

		if !activationSuccessful {
			a.emitLogRunner(eventName, "--- FALHA NA ATIVAÇÃO: NENHUM SERVIDOR KMS FUNCIONOU. ---")
		}
	}()
}


func (a *App) AjustarHoraFormatacao() {
	go func() {
		eventName := "log:ajustar:hora:formatacao"
		a.emitLogRunner(eventName, "Iniciando ajuste da hora de formatação...")

		a.runCommandAndLog(eventName, "Configurando serviço de horário (w32time) para iniciar automaticamente...", "sc", "config", "w32time", "start=auto")
		a.runCommandAndLog(eventName, "Sincronizando hora com servidor NTP (pool.ntp.br)...", "w32tm", "/config", "/syncfromflags:manual", "/manualpeerlist:\"pool.ntp.br\"", "/reliable:YES", "/update")
		a.runCommandAndLog(eventName, "Reiniciando o serviço de horário do Windows...", "net", "stop", "w32time")
		time.Sleep(1 * time.Second)
		a.runCommandAndLog(eventName, "", "net", "start", "w32time")

		now := time.Now().Unix()
		installDate := fmt.Sprintf("%d", now)
		a.runCommandAndLog(eventName, "Ajustando InstallDate no registro para o timestamp atual...", "reg", "add", `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`, "/v", "InstallDate", "/t", "REG_DWORD", "/d", installDate, "/f")
		
		a.emitLogRunner(eventName, "--- AJUSTE DE HORA CONCLUÍDO ---")
	}()
}

func (a *App) CorrigirCompartilhamentoWindows() {
	go func() {
		eventName := "log:compartilhamento"
		a.emitLogRunner(eventName, "INICIANDO CORREÇÃO DE COMPARTILHAMENTO DE REDE...")

		a.emitLogRunner(eventName, "\n--> Etapa 1/4: Configurando Serviços de Rede...")
		servicos := []string{"LanmanServer", "LanmanWorkstation", "FDResPub", "SSDPSRV", "IKEEXT", "PolicyAgent"}
		for _, s := range servicos {
			a.runCommandAndLog(eventName, "Configurando serviço: "+s, "sc", "config", s, "start=auto")
			a.runCommandAndLog(eventName, "", "net", "start", s)
		}

		a.emitLogRunner(eventName, "\n--> Etapa 2/4: Configurando Regras de Firewall do Windows...")
		a.runCommandAndLog(eventName, "Habilitando grupo 'Compartilhamento de Arquivos e Impressoras'...", "netsh", "advfirewall", "firewall", "set", "rule", "group=\"File and Printer Sharing\"", "new", "enable=Yes")
		a.runCommandAndLog(eventName, "Habilitando grupo 'Remote Service Management'...", "netsh", "advfirewall", "firewall", "set", "rule", "group=\"Remote Service Management\"", "new", "enable=yes")

		a.emitLogRunner(eventName, "\n--> Etapa 3/4: Aplicando Configurações no Registro do Windows...")
		type regChange struct {
			Path, Value, Type, Data, LogMsg string
		}
		changes := []regChange{
			{`HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters`, "AllowInsecureGuestAuth", "REG_DWORD", "1", "Habilitando logons de convidado não seguros..."},
			{`HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters`, "RequireSecuritySignature", "REG_DWORD", "0", "Ajustando política de assinatura digital (Require)..."},
			{`HKLM\SYSTEM\CurrentControlSet\Control\Print`, "RpcAuthnLevelPrivacyEnabled", "REG_DWORD", "0", "Desativando privacidade RPC estrita para impressoras..."},
			{`HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint`, "RestrictDriverInstallationToAdministrators", "REG_DWORD", "0", "Permitindo instalação de drivers de impressão..."},
			{`HKLM\SYSTEM\CurrentControlSet\Control\Lsa`, "limitblankpassworduse", "REG_DWORD", "0", "Habilitando acesso para usuários com senha em branco..."},
		}
		for _, change := range changes {
			a.runCommandAndLog(eventName, change.LogMsg, "reg", "add", change.Path, "/v", change.Value, "/t", change.Type, "/d", change.Data, "/f")
		}

		a.emitLogRunner(eventName, "\n--> Etapa 4/4: Finalizando e Aplicando Políticas...")
		a.runCommandAndLog(eventName, "Reiniciando Spooler de Impressão...", "net", "stop", "spooler")
		time.Sleep(2 * time.Second)
		a.runCommandAndLog(eventName, "", "net", "start", "spooler")
		a.runCommandAndLog(eventName, "Forçando atualização das políticas de grupo...", "gpupdate", "/force")

		a.emitLogRunner(eventName, "\n--- OPERAÇÃO CONCLUÍDA ---")
		a.emitLogRunner(eventName, "É altamente recomendável reiniciar o computador.")
		runtime.EventsEmit(a.ctx, "compartilhamento:finalizado")
	}()
}


// --- Funções Auxiliares ---
func (a *App) emitLogRunner(eventName string, mensagem string) {
	runtime.EventsEmit(a.ctx, eventName, mensagem)
}

func (a *App) runCommandAndLog(eventName, logMsg string, command string, args ...string) {
	if logMsg != "" {
		a.emitLogRunner(eventName, "--> "+logMsg)
	}
	output, err := syscmd.RunCommand("", command, args...)
	if err != nil {
		a.emitLogRunner(eventName, fmt.Sprintf("AVISO: Comando encontrou um erro (pode ser normal): %v", err))
	}
	if output != "" {
		a.emitLogRunner(eventName, strings.TrimSpace(output))
	}
}

func (a *App) instalarLicencasOffice(eventName, officePath string, patterns []string) {
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
		a.emitLogRunner(eventName, "Aviso: Diretório de licenças KMS não encontrado. Pulando esta etapa.")
		return
	}
	
	files, err := os.ReadDir(foundLicensesDir)
	if err != nil {
		a.emitLogRunner(eventName, fmt.Sprintf("Erro ao ler diretório de licenças %s: %v", foundLicensesDir, err))
		return
	}

	osppPath := filepath.Join(officePath, "ospp.vbs")
	for _, pattern := range patterns {
		re := regexp.MustCompile(pattern)
		for _, file := range files {
			if !file.IsDir() && re.MatchString(file.Name()) {
				licensePath := filepath.Join(foundLicensesDir, file.Name())
				a.runCommandAndLog(eventName, fmt.Sprintf("Instalando licença KMS: %s", file.Name()), "cscript", osppPath, "/inslic:"+licensePath)
			}
		}
	}
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