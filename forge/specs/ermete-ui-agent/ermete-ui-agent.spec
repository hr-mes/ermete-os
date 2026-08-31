Name:           ermete-ui-agent
Version:        1.0.0
Release:        3%{?dist}
Summary:        Ermete Generative UI Agent

License:        MIT


BuildArch:      noarch
Requires: python3
Requires:       python3-aiohttp

%description
Context-aware generative UI daemon for Ermete OS. Interfaces with local LLMs (Ollama) to orchestrate desktop widgets natively based on system context.

%prep
# Stub prep

%build
# Nothing to build, Python script

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname agent.py) && touch agent.py
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname SYSTEM_PROMPT.md) && touch SYSTEM_PROMPT.md
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname ermete-ui-agent.service) && touch ermete-ui-agent.service

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

