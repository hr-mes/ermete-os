$inputJson = $input | ConvertFrom-Json
if ($inputJson.toolCall.args.CommandLine -match "git commit") {
    Write-Host '{"decision": "ask", "reason": "?? ERMETE OS ZERO-TRUST: Stai per committare. Verifica che Kani e Rustfmt siano passati. La pipeline CI dipende da questo!"}'
} else {
    Write-Host '{"decision": "allow"}'
}
