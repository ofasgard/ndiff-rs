use nmap_xml_parser::NmapResults;
use nmap_xml_parser::host::Host;
use nmap_xml_parser::host::HostStatus;
use nmap_xml_parser::port::PortInfo;
use nmap_xml_parser::port::Port;
use nmap_xml_parser::host::Address;
use nmap_xml_parser::host::Hostname;

pub struct HostWrapper(pub Host);
pub struct HostStatusWrapper(pub HostStatus);
pub struct PortInfoWrapper(pub PortInfo);
pub struct AddressesWrapper(pub Vec<Address>);
pub struct HostnamesWrapper(pub Vec<Hostname>);

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

impl PartialEq for PortInfoWrapper {
	fn eq(&self, other: &PortInfoWrapper) -> bool {
		// Convert both iterators into a vector of ports.
		let mut left : Vec<&Port> = self.0.ports().collect();
		let mut right : Vec<&Port> = other.0.ports().collect();
		
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

#[derive(Debug)]
pub struct HostDiff {
	pub status: Option<(HostStatus,HostStatus)>,
	pub portinfo: Option<(PortInfo,PortInfo)>,
	pub addresses: Option<(Vec<Address>,Vec<Address>)>,
	pub hostnames: Option<(Vec<Hostname>,Vec<Hostname>)>
}

impl HostDiff {
	pub fn from_hosts(left : &Host, right : &Host) -> HostDiff {
		let status = match HostStatusWrapper(left.status.clone()) == HostStatusWrapper(right.status.clone()) {
			false => Some((left.status.clone(), right.status.clone())),
			true => None
		};
		let portinfo = match PortInfoWrapper(left.port_info.clone()) == PortInfoWrapper(right.port_info.clone()) {
			false => Some((left.port_info.clone(), right.port_info.clone())),
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
			status: status,
			portinfo: portinfo,
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
