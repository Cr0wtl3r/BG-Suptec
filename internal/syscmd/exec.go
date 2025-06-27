package syscmd

import (
	"bytes"
	"io"
	"os/exec"
	"syscall"

	"golang.org/x/text/encoding/charmap"
	"golang.org/x/text/transform"
)

func RunCommand(dir, name string, args ...string) (string, error) {
	cmd := exec.Command(name, args...)
	if dir != "" {
		cmd.Dir = dir
	}
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true}
	output, err := cmd.CombinedOutput()


	decoder := charmap.CodePage850.NewDecoder()
	reader := transform.NewReader(bytes.NewReader(output), decoder)

	utf8Bytes, readErr := io.ReadAll(reader)
	if readErr != nil {
		return string(output), err
	}

	return string(utf8Bytes), err
}
