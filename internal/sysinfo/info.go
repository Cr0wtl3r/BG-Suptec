package sysinfo

import (
	"fmt"
	"net"
	"os"
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
}

func GetInfo() (InfoSistema, error) {
	var info InfoSistema
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
		info.Processador = processorName
	}
	totalMemoryBytes := memory.TotalMemory()
	totalMemoryGB := float64(totalMemoryBytes) / (1024 * 1024 * 1024)
	info.MemoriaTotalGB = fmt.Sprintf("%.2f GB", totalMemoryGB)

	info.EnderecoMAC = "N/A"
	info.EnderecoIP = "N/A"

	interfaces, err := net.Interfaces()
	if err == nil {
		for _, i := range interfaces {
			if i.Flags&net.FlagUp != 0 && !strings.Contains(i.Flags.String(), "loopback") {
				addrs, err := i.Addrs()
				if err != nil {
					continue
				}
				for _, addr := range addrs {
					var ip net.IP
					switch v := addr.(type) {
					case *net.IPNet:
						ip = v.IP
					case *net.IPAddr:
						ip = v.IP
					}
					if ip != nil && ip.To4() != nil {
						info.EnderecoMAC = i.HardwareAddr.String()
						info.EnderecoIP = ip.String()
						goto network_info_found
					}
				}
			}
		}
	}

	network_info_found:

	return info, nil
}