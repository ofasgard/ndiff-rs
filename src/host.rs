// TODO: within a HostDiff, remove all ports that are identical i.e. tcp 80 (conn-refused) [http] => tcp 80 (conn-refused) [http]
// TODO: improve/condense the display format, the current version is just a working prototype

use std::fmt;

use nmap_xml_parser::NmapResults;
use nmap_xml_parser::host::Host;
use nmap_xml_parser::host::HostStatus;
use nmap_xml_parser::port::Port;
use nmap_xml_parser::host::Address;
use nmap_xml_parser::host::Hostname;

pub struct HostWrapper(pub Host);
pub struct HostStatusWrapper(pub HostStatus);
pub struct PortsWrapper(pub Vec<Port>);
pub struct AddressesWrapper(pub Vec<Address>);
pub struct HostnamesWrapper(pub Vec<Hostname>);

impl HostWrapper {
	fn get_title(&self) -> String {
		let host_name = match self.0.host_names().next() { Some(x) => x.name.to_string(), None => "<no hostname>".to_string() };
		let address = match self.0.addresses().next() {
			Some(addr) => match addr {
				Address::IpAddr(x) => x.to_string(),
				Address::MacAddr(x) => x.to_string()
			}
			None => "<no address>".to_string()
		};
		format!("{} ({})", host_name, address)
	}
}

#[derive(Debug)]
pub struct HostDiff {
	pub title: String,
	pub status: Option<(HostStatus,HostStatus)>,
	pub ports: Option<(Vec<Port>,Vec<Port>)>,
	pub addresses: Option<(Vec<Address>,Vec<Address>)>,
	pub hostnames: Option<(Vec<Hostname>,Vec<Hostname>)>
}

impl HostDiff {
	pub fn from_hosts(left : &Host, right : &Host) -> HostDiff {
		let title = HostWrapper(right.clone()).get_title();
	
		let status = match HostStatusWrapper(left.status.clone()) == HostStatusWrapper(right.status.clone()) {
			false => Some((left.status.clone(), right.status.clone())),
			true => None
		};
		
		let left_ports : Vec<Port> = left.port_info.ports().map(|x| x.clone()).collect();
		let right_ports : Vec<Port> = right.port_info.ports().map(|x| x.clone()).collect();
		let ports = match PortsWrapper(left_ports.clone()) == PortsWrapper(right_ports.clone()) {
			false => Some((left_ports, right_ports)),
			true => None
		};
		
		let left_addresses : Vec<Address> = left.addresses().map(|x| x.clone()).collect();
		let right_addresses : Vec<Address> = right.addresses().map(|x| x.clone()).collect();
		let addresses = match AddressesWrapper(left_addresses.clone()) == AddressesWrapper(right_addresses.clone()) {
			false => Some((left_addresses, right_addresses)),
			true => None
		};
		
		let left_hostnames : Vec<Hostname> = left.host_names().map(|x| x.clone()).collect();
		let right_hostnames : Vec<Hostname> = right.host_names().map(|x| x.clone()).collect();
		let hostnames = match HostnamesWrapper(left_hostnames.clone()) == HostnamesWrapper(right_hostnames.clone()) {
			false => Some((left_hostnames, right_hostnames)),
			true => None
		};
				
		HostDiff {
			title: title,
			status: status,
			ports: ports,
			addresses: addresses,
			hostnames: hostnames
		}
	}
}


#[derive(Debug)]
pub enum HostDelta {
	Changed(HostDiff),
	Gone(Host),
	New(Host)
}

impl HostDelta {
	pub fn from_scans(old : &NmapResults, new : &NmapResults) -> Vec<HostDelta> {
		let mut output : Vec<HostDelta> = Vec::new();
		
		// Iterate through the old scan and identify any hosts that don't exist in the new scan.
		for old_host in old.hosts() {
			if !new.hosts().any(|x| { HostWrapper(old_host.clone()) == HostWrapper(x.clone()) }) {
				let gone = HostDelta::Gone(old_host.clone());
				output.push(gone);
			}
		}
		
		// Iterate through the new scan and identify any hosts that don't exist in the old scan (eliminating them).
		let mut remaining_hosts : Vec<Host> = new.hosts().map(|x| { x.clone() }).collect();
		remaining_hosts.retain(|host| {
			if !old.hosts().any(|x| { HostWrapper(host.clone()) == HostWrapper(x.clone()) }) {
				let new = HostDelta::New(host.clone());
				output.push(new);
				return false;
			}
		
			true
		});
		
		// Iterate through the remaining "changed" hosts and generate a HostDiff for them.
		for host in remaining_hosts {
			for old_host in old.hosts() {
				if HostWrapper(host.clone()) == HostWrapper(old_host.clone()) {
					let diff = HostDiff::from_hosts(old_host, &host);
					let changed = HostDelta::Changed(diff);
					output.push(changed);
				}
			}
		}
	
		output
	}
}

// EQUALITY IMPLEMENTATIONS

impl PartialEq for HostWrapper {
	fn eq(&self, other: &HostWrapper) -> bool {
		// If any of the IP or MAC addresses match between two hosts, we consider them to be the same host.
		for self_address in self.0.addresses() {
			for other_address in other.0.addresses() {
				if self_address == other_address { return true; }
			}
		}
		
		false
	}
}

impl PartialEq for HostStatusWrapper {
	fn eq(&self, other: &HostStatusWrapper) -> bool {
		// If the state and reason is the same, we don't consider host state to have changed.
		self.0.state == other.0.state && self.0.reason == other.0.reason
	}
}

impl PartialEq for PortsWrapper {
	fn eq(&self, other: &PortsWrapper) -> bool {
		let mut left = self.0.clone();
		let mut right = other.0.clone();
		
		// Sort and compare the vectors to determine whether the port list is identical.
		left.sort_by_key(|x| { x.port_number });
		right.sort_by_key(|x| { x.port_number });
		left == right
	}
}

impl PartialEq for AddressesWrapper {
	fn eq(&self, other: &AddressesWrapper) -> bool {
		// Compare the addresses, irrespective of order, for equality.
		let mut eq = true;
		for address in &self.0 {
			if !other.0.contains(&address) { eq = false; }
		}
		eq
	}
}

impl PartialEq for HostnamesWrapper {
	fn eq(&self, other: &HostnamesWrapper) -> bool {
		// Compare the hostnames, irrespective of order, for equality.
		let mut eq = true;
		for hostname in &self.0 {
			if !other.0.contains(&hostname) { eq = false; }
		}
		eq
	}
}

// DISPLAY IMPLEMENTATIONS

impl fmt::Display for HostWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Status: {} ({})\n\n", self.0.status.state.to_string(), self.0.status.reason)?;
		
		let ports = PortsWrapper(self.0.port_info.ports().map(|x| x.clone()).collect());
		write!(f, "Ports:\n\n{}\n", ports.to_string())?;
		
		let addresses = AddressesWrapper(self.0.addresses().map(|x| x.clone()).collect());
		write!(f, "Addresses:\n\n{}\n", addresses.to_string())?;
		
		let hostnames = HostnamesWrapper(self.0.host_names().map(|x| x.clone()).collect());
		write!(f, "Hostnames:\n\n{}\n", hostnames.to_string())?;
		
		Ok(())
	}
}

impl fmt::Display for PortsWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for port in &self.0 {
			write!(f, "{} {} ({})", port.protocol.to_string(), port.port_number, port.status.reason)?;
			if let Some(serviceinfo) = &port.service_info {
				write!(f, " [{}]", serviceinfo.name)?;
			}
			write!(f, "\n")?;
		}
		Ok(())
	}
}

impl fmt::Display for AddressesWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for address in &self.0 {
			let addr_str = match address {
				Address::IpAddr(x) => x.to_string(),
				Address::MacAddr(x) => x.to_string()
			};
			write!(f, "{}\n", addr_str)?;
		}
		Ok(())
	}
}

impl fmt::Display for HostnamesWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for hostname in &self.0 {
			write!(f, "{}\n", hostname.name)?;
		}
		Ok(())
	}
}

impl fmt::Display for HostDiff {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(status) = &self.status {
			write!(f, "Status: {} ({}) => {} ({})\n\n", status.0.state.to_string(), status.0.reason, status.1.state.to_string(), status.1.reason)?;
		}
		
		if let Some(ports) = &self.ports {
			let left = PortsWrapper(ports.0.clone());
			let right = PortsWrapper(ports.1.clone());
			write!(f, "Old Ports:\n\n{}\n", left.to_string())?;
			write!(f, "New Ports:\n\n{}\n", right.to_string())?;
		}
		
		if let Some(addresses) = &self.addresses {
			let left = AddressesWrapper(addresses.0.clone());
			let right = AddressesWrapper(addresses.1.clone());
			write!(f, "Old Addresses:\n\n{}\n", left.to_string())?;
			write!(f, "New Addresses:\n\n{}\n", right.to_string())?;
		}
		
		if let Some(hostnames) = &self.hostnames {
			let left = HostnamesWrapper(hostnames.0.clone());
			let right = HostnamesWrapper(hostnames.1.clone());
			write!(f, "Old Hostnames:\n\n{}\n", left.to_string())?;
			write!(f, "New Hostnames:\n\n{}\n", right.to_string())?;
		}
		
		Ok(())
	}
}

impl fmt::Display for HostDelta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let display_str = match &self {
			HostDelta::Changed(x) => format!("Changed Host: {}\n\n{}\n", x.title, x.to_string()),
			HostDelta::Gone(x) => format!("Gone Host: {}\n\n{}\n", HostWrapper(x.clone()).get_title(), HostWrapper(x.clone()).to_string()),
			HostDelta::New(x) => format!("New Host: {}\n\n{}\n", HostWrapper(x.clone()).get_title(), HostWrapper(x.clone()).to_string())
		};
		write!(f, "{}", display_str)
	}
}
