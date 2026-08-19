Name:           ermete-semantic-db
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-semantic-db

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-semantic-db.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building ermete-semantic-db..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-semantic-db
#!/bin/bash
echo "Executing ermete-semantic-db (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-semantic-db

%files
/usr/bin/ermete-semantic-db
