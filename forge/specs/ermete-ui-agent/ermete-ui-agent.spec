Name:           ermete-ui-agent
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete Generative UI Agent

License:        MIT
Source0:        ermete-ui-agent-%{version}.tar.gz

BuildArch:      noarch
Requires:       python3 python3-requests

%description
Context-aware generative UI daemon for Ermete OS. Interfaces with local LLMs (Ollama) to orchestrate desktop widgets natively based on system context.

%prep
%autosetup

%build
# Nothing to build, Python script

%install
mkdir -p %{buildroot}/usr/libexec/ermete-ui-agent
install -m 0755 agent.py %{buildroot}/usr/libexec/ermete-ui-agent/agent.py
install -m 0644 SYSTEM_PROMPT.md %{buildroot}/usr/libexec/ermete-ui-agent/SYSTEM_PROMPT.md

mkdir -p %{buildroot}/usr/lib/systemd/user
install -m 0644 ermete-ui-agent.service %{buildroot}/usr/lib/systemd/user/ermete-ui-agent.service

%files
/usr/libexec/ermete-ui-agent/agent.py
/usr/libexec/ermete-ui-agent/SYSTEM_PROMPT.md
/usr/lib/systemd/user/ermete-ui-agent.service

%changelog
* Sun Jul 19 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of the Generative UI agent
