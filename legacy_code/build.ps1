$envFile = "./.env"

if (-not (Test-Path $envFile)) {
    Write-Error "Erro: O arquivo .env não foi encontrado em $envFile"
    Write-Error "Certifique-se de que ele contém a variável PASSWORD."
    exit 1
}

$envContent = Get-Content $envFile -Raw

$passwordHash = ($envContent | Select-String -Pattern "PASSWORD=(.*)").Matches.Groups[1].Value

if ([string]::IsNullOrEmpty($passwordHash)) {
    Write-Error "Erro: PASSWORD (hash) não encontrada no arquivo .env ou a linha está vazia."
    exit 1
}

Write-Host "Iniciando build do Wails com hash da senha injetado..."

& wails build "-ldflags=-X main.compiledPasswordHash=$passwordHash"

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build concluído com sucesso!"
} else {
    Write-Error "Erro durante o build do Wails. Código de saída: $LASTEXITCODE"
}