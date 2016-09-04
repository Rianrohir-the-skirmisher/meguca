// Webserver

package server

import (
	"fmt"
	"log"
	"net/http"

	"github.com/bakape/meguca/auth"
	"github.com/bakape/meguca/util"
)

// Base set of HTTP headers for both HTML and JSON
var vanillaHeaders = map[string]string{
	"X-Frame-Options": "sameorigin",
	"Cache-Control":   "max-age=0, must-revalidate",
	"Expires":         "Fri, 01 Jan 1990 00:00:00 GMT",
}

// Build an etag for HTML or JSON pages and check if it matches the one provided
// by the client. If yes, send 304 and return false, otherwise set headers and
// return true.
func pageEtag(res http.ResponseWriter, req *http.Request, etag string) bool {
	// If etags match, no need to rerender
	if checkClientEtag(res, req, etag) {
		return false
	}
	setHeaders(res, etag)
	return true
}

// Build the main part of the etag
func etagStart(counter int64) string {
	return "W/" + util.IDToString(counter)
}

// Check is any of the etags the client provides in the "If-None-Match" header
// match the generated etag. If yes, write 304 and return true.
func checkClientEtag(
	res http.ResponseWriter,
	req *http.Request,
	etag string,
) bool {
	if etag == req.Header.Get("If-None-Match") {
		res.WriteHeader(304)
		return true
	}
	return false
}

// Write a []byte to the client
func writeData(w http.ResponseWriter, r *http.Request, data []byte) {
	_, err := w.Write(data)
	if err != nil {
		logError(r, err)
	}
}

// Log an error together with the client's IP
func logError(r *http.Request, err interface{}) {
	log.Printf("server: %s: %s", auth.GetIP(r), err)
}

// Set HTTP headers to the response object
func setHeaders(res http.ResponseWriter, etag string) {
	head := res.Header()
	for key, val := range vanillaHeaders {
		head.Set(key, val)
	}
	head.Set("ETag", etag)
}

// Text-only 404 response
func text404(w http.ResponseWriter) {
	http.Error(w, "404 Not found", 404)
}

// Text-only 500 response
func text500(w http.ResponseWriter, r *http.Request, err interface{}) {
	http.Error(w, fmt.Sprintf("500 %s", err), 500)
	logError(r, err)
}