/*
 * The Accept-Encoding of OBS's outgoing HTTP calls (file-updater, the
 * frontend's RemoteTextThread). Header-only so the C and C++ call sites share
 * it without a new link dependency.
 */

#pragma once

#include <curl/curl.h>

/* The CURLOPT_ACCEPT_ENCODING value for the libcurl described by info. */
static inline const char *obs_curl_accept_encoding(const curl_version_info_data *info)
{
	(void)info;
	return "gzip, deflate";
}
