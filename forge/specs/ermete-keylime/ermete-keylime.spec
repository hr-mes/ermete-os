Name:           ermete-keylime
Version:        1.0
Release:        1%{?dist}
Summary:        Ermete OS Keylime Agent Configuration
License:        GPLv3+
URL:            https://github.com/ermete-os
Source0:        99-ermete.conf

Requires:       keylime-agent
Requires:       tpm2-tools
BuildArch:      noarch

%description
Pacchetto di configurazione per l'agente Keylime in Ermete OS.
Implementa Remote Attestation (Fase 3) bindando misurazioni TPM
e sigillando l'enclave di sicurezza.

%prep
%setup -q -c -T
cp %{SOURCE0} .

%build

%install
mkdir -p %{buildroot}/etc/keylime/agent.conf.d/
install -m 0644 99-ermete.conf %{buildroot}/etc/keylime/agent.conf.d/99-ermete.conf

%files
%defattr(-,root,root,-)
%dir /etc/keylime/agent.conf.d
%config(noreplace) /etc/keylime/agent.conf.d/99-ermete.conf

%changelog
* Mon Aug 03 2026 Ermete Core <core@ermete.os> - 1.0-1
- Initial release for Phase 3
