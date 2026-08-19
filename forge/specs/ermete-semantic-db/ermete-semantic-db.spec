Name:           ermete-semantic-db
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-semantic-db

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-semantic-db.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-semantic-db) && touch %{buildroot}/usr/bin/ermete-semantic-db


%files
/usr/bin/ermete-semantic-db
