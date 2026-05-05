use std::net::Ipv6Addr;
use std::str::FromStr;

const UINT32_MAX: u32 = u32::MAX;
const MAX_GENERATED_ADDRESSES: u128 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpVersion {
    Ipv4,
    Ipv6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tone {
    Blue,
    Yellow,
    Green,
    Magenta,
}

impl Tone {
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Blue => 0,
            Self::Yellow => 1,
            Self::Green => 2,
            Self::Magenta => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalcLine {
    pub label: &'static str,
    pub value: String,
    pub tone: Tone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Calculation {
    pub lines: Vec<CalcLine>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IpRange {
    start: u128,
    end: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Ipv4NetworkInput {
    ip: u32,
    prefix: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Ipv4Subnet {
    ip: u32,
    prefix: u8,
    mask: u32,
    wildcard: u32,
    network: u32,
    broadcast: u32,
    first_host: u32,
    last_host: u32,
    total_addresses: u64,
    usable_hosts: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Ipv6Subnet {
    ip: u128,
    prefix: u8,
    network: u128,
    last_address: u128,
    total_addresses: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IpCount {
    Forward(u128),
    Backward(u128),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Forward,
    Backward,
}

pub fn calculate(input: &str, version: IpVersion) -> Result<Calculation, String> {
    match version {
        IpVersion::Ipv4 => calculate_ipv4(input),
        IpVersion::Ipv6 => calculate_ipv6(input),
    }
}

pub fn calculate_range_addresses(
    input: &str,
    range_input: &str,
    version: IpVersion,
) -> Result<Vec<String>, String> {
    match version {
        IpVersion::Ipv4 => calculate_ipv4_range_addresses(input, range_input),
        IpVersion::Ipv6 => calculate_ipv6_range_addresses(input, range_input),
    }
}

#[cfg(test)]
pub fn calculate_lines(input: &str, version: IpVersion) -> Result<Vec<CalcLine>, String> {
    calculate(input, version).map(|calculation| calculation.lines)
}

fn calculate_ipv4(input: &str) -> Result<Calculation, String> {
    let parsed = parse_ipv4_network_input(input)?;
    let data = calculate_subnet(parsed.ip, parsed.prefix)?;

    let lines = vec![
        line(
            "CIDR notation",
            format!("{}/{}", format_ipv4(data.network), data.prefix),
            Tone::Blue,
        ),
        line("Network address", format_ipv4(data.network), Tone::Blue),
        line("Broadcast address", format_ipv4(data.broadcast), Tone::Blue),
        line("Subnet mask", format_ipv4(data.mask), Tone::Yellow),
        line("Wildcard mask", format_ipv4(data.wildcard), Tone::Yellow),
        line(
            "Usable host range",
            format!(
                "{} - {}",
                format_ipv4(data.first_host),
                format_ipv4(data.last_host)
            ),
            Tone::Green,
        ),
        line(
            "Usable number of hosts",
            format_with_commas(data.usable_hosts),
            Tone::Green,
        ),
        line(
            "Total number of addresses",
            format_with_commas(data.total_addresses),
            Tone::Green,
        ),
        line(
            "IP Type",
            format!("{} / {}", get_ip_type(data.ip), get_network_class(data.ip)),
            Tone::Magenta,
        ),
    ];
    Ok(Calculation { lines })
}

fn calculate_ipv4_range_addresses(input: &str, range_input: &str) -> Result<Vec<String>, String> {
    let parsed = parse_ipv4_network_input(input)?;
    let data = calculate_subnet(parsed.ip, parsed.prefix)?;
    let range = parse_ip_range(range_input, Some(255))?;

    ipv4_range_addresses(&data, parsed.ip, range)
}

fn calculate_ipv6(input: &str) -> Result<Calculation, String> {
    let (ip, prefix) = parse_ipv6_cidr(input)?;
    let data = calculate_ipv6_subnet(ip, prefix)?;

    let lines = vec![
        line("Input address", format_ipv6(data.ip), Tone::Blue),
        line(
            "CIDR notation",
            format!("{}/{}", format_ipv6(data.network), data.prefix),
            Tone::Blue,
        ),
        line("Network prefix", format_ipv6(data.network), Tone::Blue),
        line("Prefix length", format!("/{}", data.prefix), Tone::Yellow),
        line("First address", format_ipv6(data.network), Tone::Green),
        line("Last address", format_ipv6(data.last_address), Tone::Green),
        line(
            "Total number of addresses",
            data.total_addresses.clone(),
            Tone::Green,
        ),
        line("IP Type", get_ipv6_type(data.ip).to_owned(), Tone::Magenta),
    ];
    Ok(Calculation { lines })
}

fn calculate_ipv6_range_addresses(input: &str, range_input: &str) -> Result<Vec<String>, String> {
    let (ip, prefix) = parse_ipv6_cidr(input)?;
    let data = calculate_ipv6_subnet(ip, prefix)?;
    let range = parse_ip_count(range_input)?;

    ipv6_range_addresses(&data, range)
}

fn line(label: &'static str, value: String, tone: Tone) -> CalcLine {
    CalcLine { label, value, tone }
}

fn parse_ipv4_network_input(input: &str) -> Result<Ipv4NetworkInput, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(
            "Enter IPv4 as CIDR, address + subnet mask, or address + wildcard mask.".to_owned(),
        );
    }

    if trimmed.contains('/') {
        let (ip_part, prefix_part) = split_once_strict(trimmed, '/')
            .ok_or_else(|| "CIDR input must use address/prefix notation.".to_owned())?;
        let ip = parse_ipv4(ip_part)?;
        let prefix = parse_ipv4_prefix(prefix_part)?;

        return Ok(Ipv4NetworkInput { ip, prefix });
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(
            "Use 192.168.1.0/24, 192.168.1.0 255.255.255.0, or 192.168.1.0 0.0.15.255.".to_owned(),
        );
    }

    let ip = parse_ipv4(parts[0])?;
    let mask_or_wildcard = parse_ipv4(parts[1])?;

    if let Some(prefix) = mask_to_prefix(mask_or_wildcard) {
        return Ok(Ipv4NetworkInput { ip, prefix });
    }

    if let Some(prefix) = wildcard_to_prefix(mask_or_wildcard) {
        return Ok(Ipv4NetworkInput { ip, prefix });
    }

    Err("The second value is not a contiguous subnet mask or wildcard mask.".to_owned())
}

fn parse_ipv4(input: &str) -> Result<u32, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Enter an IPv4 address.".to_owned());
    }

    let parts: Vec<&str> = trimmed.split('.').collect();

    if parts.len() != 4 {
        return Err("IPv4 addresses must contain four octets.".to_owned());
    }

    let mut octets = [0u8; 4];

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("Each octet must be a number from 0 to 255.".to_owned());
        }

        let value: u16 = part
            .parse()
            .map_err(|_| "Each octet must be a number from 0 to 255.".to_owned())?;

        if value > 255 {
            return Err("Each octet must be a number from 0 to 255.".to_owned());
        }

        octets[index] = value as u8;
    }

    Ok(octets_to_number(octets))
}

fn parse_ipv4_prefix(input: &str) -> Result<u8, String> {
    parse_prefix(input, 32, "Prefix length must be an integer from 0 to 32.")
}

fn parse_ipv6_prefix(input: &str) -> Result<u8, String> {
    parse_prefix(
        input,
        128,
        "IPv6 prefix length must be an integer from 0 to 128.",
    )
}

fn parse_prefix(input: &str, max: u8, error: &str) -> Result<u8, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error.to_owned());
    }

    let value: u16 = trimmed.parse().map_err(|_| error.to_owned())?;

    if value > max as u16 {
        return Err(error.to_owned());
    }

    Ok(value as u8)
}

fn split_once_strict(input: &str, separator: char) -> Option<(&str, &str)> {
    let (left, right) = input.split_once(separator)?;

    if right.contains(separator) {
        return None;
    }

    Some((left, right))
}

fn parse_ip_range(input: &str, max_value: Option<u128>) -> Result<IpRange, String> {
    let (start, end) = split_once_strict(input, '-')
        .ok_or_else(|| "IP range must use start-end, for example 1-20.".to_owned())?;

    let start = parse_ip_range_value(start, max_value)?;
    let end = parse_ip_range_value(end, max_value)?;

    if start > end {
        return Err("IP range start must be less than or equal to the end.".to_owned());
    }

    Ok(IpRange { start, end })
}

fn parse_ip_count(input: &str) -> Result<IpCount, String> {
    let trimmed = input.trim();
    let (direction, count_input) = if let Some(count_input) = trimmed.strip_prefix('-') {
        (Direction::Backward, count_input)
    } else if let Some(count_input) = trimmed.strip_prefix('+') {
        (Direction::Forward, count_input)
    } else {
        (Direction::Forward, trimmed)
    };

    if count_input.is_empty() || !count_input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("IPv6 address count must be a decimal number.".to_owned());
    }

    let count: u128 = count_input
        .parse()
        .map_err(|_| "IPv6 address count must be a decimal number.".to_owned())?;

    if count == 0 {
        return Err("IPv6 address count must start at 1.".to_owned());
    }

    Ok(match direction {
        Direction::Forward => IpCount::Forward(count),
        Direction::Backward => IpCount::Backward(count),
    })
}

fn parse_ip_range_value(input: &str, max_value: Option<u128>) -> Result<u128, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("IP range values must be decimal numbers.".to_owned());
    }

    let value: u128 = trimmed
        .parse()
        .map_err(|_| "IP range values must be decimal numbers.".to_owned())?;

    if value == 0 {
        return Err("IP range values must start at 1.".to_owned());
    }

    if let Some(max_value) = max_value
        && value > max_value
    {
        return Err(format!("IP range values must be from 1 to {max_value}."));
    }

    Ok(value)
}

fn range_capacity(range: IpRange) -> Result<usize, String> {
    let count = range.end - range.start + 1;

    usize::try_from(count).map_err(|_| "IP range is too large to generate.".to_owned())
}

fn octets_to_number(octets: [u8; 4]) -> u32 {
    ((octets[0] as u32) << 24)
        | ((octets[1] as u32) << 16)
        | ((octets[2] as u32) << 8)
        | octets[3] as u32
}

fn format_ipv4(value: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (value >> 24) & 255,
        (value >> 16) & 255,
        (value >> 8) & 255,
        value & 255
    )
}

fn prefix_to_mask(prefix: u8) -> Result<u32, String> {
    if prefix > 32 {
        return Err("Prefix length must be an integer from 0 to 32.".to_owned());
    }

    if prefix == 0 {
        return Ok(0);
    }

    Ok(UINT32_MAX << (32 - prefix))
}

fn prefix_to_wildcard(prefix: u8) -> Result<u32, String> {
    Ok(!prefix_to_mask(prefix)?)
}

fn mask_to_prefix(mask: u32) -> Option<u8> {
    let mut prefix = 0;
    let mut seen_zero = false;

    for bit in (0..32).rev() {
        let is_one = ((mask >> bit) & 1) == 1;

        if is_one && seen_zero {
            return None;
        }

        if is_one {
            prefix += 1;
        } else {
            seen_zero = true;
        }
    }

    Some(prefix)
}

fn wildcard_to_prefix(wildcard: u32) -> Option<u8> {
    mask_to_prefix(!wildcard)
}

fn calculate_subnet(ip: u32, prefix: u8) -> Result<Ipv4Subnet, String> {
    let mask = prefix_to_mask(prefix)?;
    let wildcard = prefix_to_wildcard(prefix)?;
    let network = ip & mask;
    let broadcast = network | wildcard;
    let host_bits = 32 - prefix;
    let total_addresses = 1u64 << host_bits;
    let usable_hosts = get_usable_host_count(prefix);

    Ok(Ipv4Subnet {
        ip,
        prefix,
        mask,
        wildcard,
        network,
        broadcast,
        first_host: get_first_host(network, broadcast, prefix),
        last_host: get_last_host(network, broadcast, prefix),
        total_addresses,
        usable_hosts,
    })
}

fn ipv4_range_addresses(
    data: &Ipv4Subnet,
    input_ip: u32,
    range: IpRange,
) -> Result<Vec<String>, String> {
    let block_base = input_ip & !0xff;
    let mut addresses = Vec::with_capacity(range_capacity(range)?);

    for last_number in range.start..=range.end {
        let candidate = block_base | last_number as u32;

        if candidate < data.network || candidate > data.broadcast {
            return Err("The requested IPv4 IP range is outside the current network.".to_owned());
        }

        addresses.push(format_ipv4(candidate));
    }

    Ok(addresses)
}

fn get_usable_host_count(prefix: u8) -> u64 {
    if prefix == 32 {
        return 1;
    }

    if prefix == 31 {
        return 2;
    }

    let host_bits = 32 - prefix;
    (1u64 << host_bits) - 2
}

fn get_first_host(network: u32, broadcast: u32, prefix: u8) -> u32 {
    if prefix >= 31 {
        return network;
    }

    (network + 1).min(broadcast)
}

fn get_last_host(network: u32, broadcast: u32, prefix: u8) -> u32 {
    if prefix >= 31 {
        return broadcast;
    }

    broadcast.saturating_sub(1).max(network)
}

fn get_network_class(ip: u32) -> &'static str {
    let first = (ip >> 24) & 255;

    if first <= 127 {
        return "A";
    }

    if first <= 191 {
        return "B";
    }

    if first <= 223 {
        return "C";
    }

    if first <= 239 {
        return "D (multicast)";
    }

    "E (reserved)"
}

fn get_ip_type(ip: u32) -> &'static str {
    let first = (ip >> 24) & 255;
    let second = (ip >> 16) & 255;

    if first == 10 {
        return "Private";
    }

    if first == 172 && (16..=31).contains(&second) {
        return "Private";
    }

    if first == 192 && second == 168 {
        return "Private";
    }

    if first == 127 {
        return "Loopback";
    }

    if first == 169 && second == 254 {
        return "Link-local";
    }

    if (224..=239).contains(&first) {
        return "Multicast";
    }

    if first >= 240 {
        return "Reserved";
    }

    if first == 0 {
        return "Current network";
    }

    "Public"
}

fn parse_ipv6_cidr(input: &str) -> Result<(u128, u8), String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Enter an IPv6 CIDR block, for example 2001:db8::1/64.".to_owned());
    }

    let (ip_part, prefix_part) = split_once_strict(trimmed, '/')
        .ok_or_else(|| "CIDR input must use address/prefix notation.".to_owned())?;

    if ip_part.contains('%') {
        return Err("IPv6 zone identifiers are not supported.".to_owned());
    }

    let ip = Ipv6Addr::from_str(ip_part)
        .map(u128::from)
        .map_err(|_| "IPv6 addresses must contain eight hextets or use :: shorthand.".to_owned())?;
    let prefix = parse_ipv6_prefix(prefix_part)?;

    Ok((ip, prefix))
}

fn calculate_ipv6_subnet(ip: u128, prefix: u8) -> Result<Ipv6Subnet, String> {
    let mask = prefix_to_ipv6_mask(prefix)?;
    let network = ip & mask;
    let last_address = network | !mask;
    let host_bits = 128 - prefix as u16;

    Ok(Ipv6Subnet {
        ip,
        prefix,
        network,
        last_address,
        total_addresses: pow2_decimal(host_bits as u32),
    })
}

fn ipv6_range_addresses(data: &Ipv6Subnet, count: IpCount) -> Result<Vec<String>, String> {
    let (start, count) = match count {
        IpCount::Forward(count) => (data.ip, count),
        IpCount::Backward(count) => (
            data.ip.checked_sub(count - 1).ok_or_else(|| {
                "The requested IPv6 IP range is outside the current network.".to_owned()
            })?,
            count,
        ),
    };
    if count > MAX_GENERATED_ADDRESSES {
        return Err(format!(
            "IP range is too large to generate. Limit is {MAX_GENERATED_ADDRESSES} addresses."
        ));
    }
    let capacity =
        usize::try_from(count).map_err(|_| "IP range is too large to generate.".to_owned())?;
    let mut addresses = Vec::with_capacity(capacity);

    for offset in 0..count {
        let candidate = start.checked_add(offset).ok_or_else(|| {
            "The requested IPv6 IP range is outside the current network.".to_owned()
        })?;

        if candidate < data.network || candidate > data.last_address {
            return Err("The requested IPv6 IP range is outside the current network.".to_owned());
        }

        addresses.push(format_ipv6(candidate));
    }

    Ok(addresses)
}

fn prefix_to_ipv6_mask(prefix: u8) -> Result<u128, String> {
    if prefix > 128 {
        return Err("IPv6 prefix length must be an integer from 0 to 128.".to_owned());
    }

    if prefix == 0 {
        return Ok(0);
    }

    Ok(u128::MAX << (128 - prefix))
}

fn format_ipv6(value: u128) -> String {
    Ipv6Addr::from(value).to_string()
}

fn get_ipv6_type(ip: u128) -> &'static str {
    if ip == 1 {
        return "Loopback";
    }

    if ip == 0 {
        return "Unspecified";
    }

    if ipv6_in_prefix(ip, "fc00::", 7) {
        return "Unique local";
    }

    if ipv6_in_prefix(ip, "fe80::", 10) {
        return "Link-local";
    }

    if ipv6_in_prefix(ip, "ff00::", 8) {
        return "Multicast";
    }

    if ipv6_in_prefix(ip, "2001:db8::", 32) {
        return "Documentation";
    }

    "Global unicast"
}

fn ipv6_in_prefix(ip: u128, network: &str, prefix: u8) -> bool {
    let network = u128::from(Ipv6Addr::from_str(network).expect("static IPv6 prefix is valid"));
    let mask = prefix_to_ipv6_mask(prefix).expect("static IPv6 prefix length is valid");

    (ip & mask) == (network & mask)
}

fn format_with_commas<T: ToString>(value: T) -> String {
    let raw = value.to_string();
    let mut formatted = String::with_capacity(raw.len() + raw.len() / 3);

    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }

        formatted.push(ch);
    }

    formatted.chars().rev().collect()
}

fn pow2_decimal(bits: u32) -> String {
    let mut digits = vec![1u8];

    for _ in 0..bits {
        let mut carry = 0u8;

        for digit in &mut digits {
            let value = *digit * 2 + carry;
            *digit = value % 10;
            carry = value / 10;
        }

        while carry > 0 {
            digits.push(carry % 10);
            carry /= 10;
        }
    }

    let raw: String = digits
        .iter()
        .rev()
        .map(|digit| char::from(b'0' + *digit))
        .collect();

    format_with_commas(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_for(lines: &[CalcLine], label: &str) -> String {
        lines
            .iter()
            .find(|line| line.label == label)
            .map(|line| line.value.clone())
            .expect("line exists")
    }

    #[test]
    fn lists_ipv4_range_addresses() {
        let addresses =
            calculate_range_addresses("192.168.1.1/22", "1-20", IpVersion::Ipv4).unwrap();

        assert_eq!(addresses.len(), 20);
        assert_eq!(addresses.first().unwrap(), "192.168.1.1");
        assert_eq!(addresses.last().unwrap(), "192.168.1.20");
    }

    #[test]
    fn lists_ipv6_range_addresses() {
        let addresses = calculate_range_addresses("fd00::1/64", "+20", IpVersion::Ipv6).unwrap();

        assert_eq!(addresses.len(), 20);
        assert_eq!(addresses.first().unwrap(), "fd00::1");
        assert_eq!(addresses.last().unwrap(), "fd00::14");
    }

    #[test]
    fn lists_ipv6_range_addresses_before_input() {
        let addresses = calculate_range_addresses("fd00::14/64", "-20", IpVersion::Ipv6).unwrap();

        assert_eq!(addresses.len(), 20);
        assert_eq!(addresses.first().unwrap(), "fd00::1");
        assert_eq!(addresses.last().unwrap(), "fd00::14");
    }

    #[test]
    fn rejects_oversized_ipv6_range_addresses() {
        let error = calculate_range_addresses("fd00::1/64", "+65537", IpVersion::Ipv6)
            .expect_err("range is rejected");

        assert_eq!(
            error,
            "IP range is too large to generate. Limit is 65536 addresses."
        );
    }

    #[test]
    fn parses_and_formats_ipv4_networks() {
        let lines = calculate_lines("192.168.1.1/22", IpVersion::Ipv4).unwrap();

        assert_eq!(value_for(&lines, "CIDR notation"), "192.168.0.0/22");
        assert_eq!(value_for(&lines, "Broadcast address"), "192.168.3.255");
        assert_eq!(value_for(&lines, "Subnet mask"), "255.255.252.0");
        assert_eq!(value_for(&lines, "Wildcard mask"), "0.0.3.255");
        assert_eq!(value_for(&lines, "Usable number of hosts"), "1,022");
    }

    #[test]
    fn parses_mask_and_wildcard_ipv4_inputs() {
        let mask = calculate_lines("192.168.1.1 255.255.252.0", IpVersion::Ipv4).unwrap();
        let wildcard = calculate_lines("192.168.1.1 0.0.3.255", IpVersion::Ipv4).unwrap();

        assert_eq!(value_for(&mask, "CIDR notation"), "192.168.0.0/22");
        assert_eq!(value_for(&wildcard, "CIDR notation"), "192.168.0.0/22");
    }

    #[test]
    fn calculates_ipv6_networks() {
        let lines = calculate_lines("2001:db8::1/64", IpVersion::Ipv6).unwrap();

        assert_eq!(value_for(&lines, "Input address"), "2001:db8::1");
        assert_eq!(value_for(&lines, "CIDR notation"), "2001:db8::/64");
        assert_eq!(value_for(&lines, "Network prefix"), "2001:db8::");
        assert_eq!(value_for(&lines, "Prefix length"), "/64");
        assert_eq!(value_for(&lines, "First address"), "2001:db8::");
        assert_eq!(
            value_for(&lines, "Last address"),
            "2001:db8::ffff:ffff:ffff:ffff"
        );
        assert_eq!(
            value_for(&lines, "Total number of addresses"),
            "18,446,744,073,709,551,616"
        );
        assert_eq!(value_for(&lines, "IP Type"), "Documentation");
    }

    #[test]
    fn normalizes_ipv6_input_address() {
        let lines = calculate_lines(
            "240E:0108:11E8:0001:0104:001C:11.64.30.207/96",
            IpVersion::Ipv6,
        )
        .unwrap();

        assert_eq!(
            value_for(&lines, "Input address"),
            "240e:108:11e8:1:104:1c:b40:1ecf"
        );
        assert_eq!(
            value_for(&lines, "CIDR notation"),
            "240e:108:11e8:1:104:1c::/96"
        );
    }
}
