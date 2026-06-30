package sysinfo

import (
	"BG-SupTec/internal/syscmd"
	"fmt"
	"net"
	"os"
	"regexp"
	"strings"

	"github.com/pbnjay/memory"
	"golang.org/x/sys/windows/registry"
)

type InfoSistema struct {
	NomeComputador  string `json:"nomeComputador"`
	VersaoWindows   string `json:"versaoWindows"`
	EdicaoWindows   string `json:"edicaoWindows"`
	BuildWindows    string `json:"buildWindows"`
	Processador     string `json:"processador"`
	MemoriaTotalGB  string `json:"memoriaTotalGB"`
	EnderecoMAC     string `json:"enderecoMAC"`
	EnderecoIP      string `json:"enderecoIP"`
	MascaraRede     string `json:"mascaraRede"`
	GatewayPadrao   string `json:"gatewayPadrao"`
	DNSPrimario     string `json:"dnsPrimario"`
	DNSSecundario   string `json:"dnsSecundario"`
	InterfaceAtiva  string `json:"interfaceAtiva"`
}

func GetInfo() (InfoSistema, error) {
	var info InfoSistema
	var err error
	info.NomeComputador, _ = os.Hostname()

	key, err := registry.OpenKey(registry.LOCAL_MACHINE, `SOFTWARE\Microsoft\Windows NT\CurrentVersion`, registry.QUERY_VALUE)
	if err == nil {
		defer key.Close()
		productName, _, _ := key.GetStringValue("ProductName")
		displayVersion, _, _ := key.GetStringValue("DisplayVersion")
		currentBuild, _, _ := key.GetStringValue("CurrentBuild")
		info.EdicaoWindows = productName
		info.VersaoWindows = displayVersion
		info.BuildWindows = currentBuild
	}

	key, err = registry.OpenKey(registry.LOCAL_MACHINE, `HARDWARE\DESCRIPTION\System\CentralProcessor\0`, registry.QUERY_VALUE)
	if err == nil {
		defer key.Close()
		processorName, _, _ := key.GetStringValue("ProcessorNameString")
		info.Processador = strings.TrimSpace(processorName)
	}


	totalMemoryBytes := memory.TotalMemory()
	totalMemoryGB := float64(totalMemoryBytes) / (1024 * 1024 * 1024)
	info.MemoriaTotalGB = fmt.Sprintf("%.2f GB", totalMemoryGB)

	err = getNetworkInfoFromPrincipalInterface(&info)
	if err != nil {
		info.EnderecoMAC = "N/A"
		info.EnderecoIP = "N/A"
		info.MascaraRede = "N/A"
		info.GatewayPadrao = "N/A"
		info.DNSPrimario = "N/A"
		info.DNSSecundario = "N/A"
		info.InterfaceAtiva = ""
		return info, fmt.Errorf("erro ao obter informações de rede: %v", err)
	}

	return info, nil
}

func getNetworkInfoFromPrincipalInterface(info *InfoSistema) error {
	cmd := `
		$netAdapter = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | ForEach-Object {
			$ipConfig = Get-NetIPConfiguration -InterfaceAlias $_.Name
			if ($ipConfig.IPv4DefaultGateway) {
				return $_
			}
		} | Select-Object -First 1

		if ($netAdapter) {
			$ipConfig = Get-NetIPConfiguration -InterfaceAlias $netAdapter.Name
			$dnsServers = (Get-DnsClientServerAddress -InterfaceAlias $netAdapter.Name -AddressFamily IPv4).ServerAddresses

			@{
				InterfaceAlias = $netAdapter.Name
				MacAddress = $netAdapter.MacAddress
				IPv4Address = $ipConfig.IPv4Address.IPAddress
				PrefixLength = $ipConfig.IPv4Address.PrefixLength
				DefaultGateway = $ipConfig.IPv4DefaultGateway.NextHop
				DNSServers = $dnsServers
			} | ConvertTo-Json -Compress -Depth 3
		}
	`
	output, err := syscmd.RunCommand("", "powershell", "-NoProfile", "-Command", cmd)
	if err != nil {
		return fmt.Errorf("falha ao executar PowerShell para obter info de rede: %v - %s", err, output)
	}

	output = strings.TrimSpace(output)
	if output == "" || !strings.HasPrefix(output, "{") {
		return fmt.Errorf("nenhuma interface de rede ativa com gateway encontrada")
	}

	info.InterfaceAtiva = extractJSONValue(output, "InterfaceAlias")
	info.EnderecoMAC = strings.ReplaceAll(extractJSONValue(output, "MacAddress"), "-", ":")
	info.EnderecoIP = extractJSONValue(output, "IPv4Address")
	info.GatewayPadrao = extractJSONValue(output, "DefaultGateway")

	prefixLength := extractJSONValue(output, "PrefixLength")
	if prefixLength != "" {
		mask, err := prefixToMask(prefixLength)
		if err == nil {
			info.MascaraRede = mask
		} else {
			info.MascaraRede = "Inválida"
		}
	} else {
		info.MascaraRede = "N/A"
	}

	dnsServersStr := extractJSONValue(output, "DNSServers")
	dnsServersStr = strings.Trim(dnsServersStr, "[] ")

	info.DNSPrimario = "N/A"
	info.DNSSecundario = "N/A"

	if dnsServersStr != "" {
		re := regexp.MustCompile(`"([^"]+)"`)
		matches := re.FindAllStringSubmatch(dnsServersStr, -1)

		if len(matches) > 0 {
			info.DNSPrimario = matches[0][1]
		}
		if len(matches) > 1 {
			info.DNSSecundario = matches[1][1]
		}
	}

	return nil
}

func prefixToMask(prefix string) (string, error) {
	prefixInt, err := Atoi(prefix)
	if err != nil {
		return "", fmt.Errorf("prefixo inválido: %s", prefix)
	}

	mask := net.CIDRMask(prefixInt, 32)
	if mask == nil {
		return "", fmt.Errorf("não foi possível criar máscara para o prefixo: %d", prefixInt)
	}

	return fmt.Sprintf("%d.%d.%d.%d", mask[0], mask[1], mask[2], mask[3]), nil
}

func Atoi(s string) (int, error) {
	var n int
	for _, ch := range s {
		if ch < '0' || ch > '9' {
			return 0, fmt.Errorf("caractere inválido em string numérica: %c", ch)
		}
		n = n*10 + int(ch-'0')
	}
	return n, nil
}


func extractJSONValue(jsonStr, key string) string {
	re := regexp.MustCompile(fmt.Sprintf(`"%s":\s*("([^"]*)"|\[.*?\]|[\d\.]+)`, key))
	matches := re.FindStringSubmatch(jsonStr)

	if len(matches) > 2 && matches[2] != "" {
		return matches[2]
	}
	if len(matches) > 1 {
		val := strings.TrimSuffix(matches[1], "}")
		val = strings.TrimSuffix(val, ",")
		return strings.TrimSpace(val)
	}
	return ""
}