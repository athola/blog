#!/bin/sh
## Generate a new certificate
TLSOUTDIR="self_signed_certs"
TLSNAME="blog"
DAYSVALID=365
DOMAIN="localhost"
LOCATION="San Antonio"
COUNTRY="US"
MYNAME="$(basename "${0}")"

validate_openssl() {
	openssl help -- >/dev/null 2<&- ||\
		{
			perr "OpenSSL is required for this utility to work";
			exit 2
		}
}

validate_outdir() {
	if [ ! -d "${TLSOUTDIR}" ]
	then
		mkdir -pm 1775 "${TLSOUTDIR}" >/dev/null 2<&- ||\
			{
				perr "${TLSOUTDIR} missing! Attempting fallback...";
				touch . >/dev/null 2<&- ||\
					{
						perr "Cannot write to ${PWD}! Exiting...";
						exit 1;
					}
				## We can write to current directory
				TLSOUTDIR="${PWD}";
			}
	fi
}

generate_certificate() {
	echo "Setting up self-signed certificates for ${DOMAIN}"
	openssl req -x509 \
		-sha256 -days "${DAYSVALID}" \
		-nodes \
		-newkey rsa:2048 \
		-subj "/CN=${DOMAIN}/C=${COUNTRY}/L=${LOCATION}" \
		-keyout "${TLSOUTDIR}/${TLSNAME}.key"\
		-out "${TLSOUTDIR}/${TLSNAME}.pem"
}

perr() {
	printf "[%s]: %s\n" "${MYNAME}" "${@}" 1>&2
}

main() {
	while getopts ":h:n:D:d:c:o:l:" opt
	do
		case ${opt} in
			"h") usage ;;
			"n") TLSNAME="${OPTARG}" ;;
			"o") TLSOUTDIR="${OPTARG}" ;;
			"d") DAYSVALID=$(( OPTARG + 0 )) ;;
			"l") LOCATION="${OPTARG}" ;;
			"c") COUNTRY="${OPTARG}" ;;
			"D") DOMAIN="${OPTARG}" ;;
			*) usage ;;
		esac
	done
	validate_openssl
	validate_outdir
	generate_certificate
}

usage() {
	printf "[%s]: Automatically generate self-signed certificates for ${DOMAIN}\n" "${MYNAME}"
	printf "\t-h\tThis help message\n\t-n\tBasename of the cert/key\n\t-o\tOutput directory\n"
	printf "\t-d\tNumber of days for which the cert is valid\n\t-D\tDomain name for cert\n"
	printf "\t-c\tCountry abbrevation in which the cert is generated\n\t-l\tLocation (city)\n"
	## last line
	exit 0
}

main "${@}"
