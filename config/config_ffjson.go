// DO NOT EDIT!
// Code generated by ffjson <https://github.com/pquerna/ffjson>
// source: config.go
// DO NOT EDIT!

package config

import (
	"encoding/base64"
	fflib "github.com/pquerna/ffjson/fflib/v1"
	"reflect"
)

func (mj *BoardConfContainer) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *BoardConfContainer) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	buf.WriteString(`{"JSON":`)
	if mj.JSON != nil {
		buf.WriteString(`"`)
		{
			enc := base64.NewEncoder(base64.StdEncoding, buf)
			enc.Write(reflect.Indirect(reflect.ValueOf(mj.JSON)).Bytes())
			enc.Close()
		}
		buf.WriteString(`"`)
	} else {
		buf.WriteString(`null`)
	}
	buf.WriteString(`,"Hash":`)
	fflib.WriteJsonString(buf, string(mj.Hash))
	buf.WriteString(`,"id":`)
	fflib.WriteJsonString(buf, string(mj.ID))
	buf.WriteString(`,"eightball":`)
	if mj.Eightball != nil {
		buf.WriteString(`[`)
		for i, v := range mj.Eightball {
			if i != 0 {
				buf.WriteString(`,`)
			}
			fflib.WriteJsonString(buf, string(v))
		}
		buf.WriteString(`]`)
	} else {
		buf.WriteString(`null`)
	}
	if mj.CodeTags {
		buf.WriteString(`,"codeTags":true`)
	} else {
		buf.WriteString(`,"codeTags":false`)
	}
	buf.WriteString(`,"title":`)
	fflib.WriteJsonString(buf, string(mj.Title))
	buf.WriteString(`,"notice":`)
	fflib.WriteJsonString(buf, string(mj.Notice))
	buf.WriteString(`,"rules":`)
	fflib.WriteJsonString(buf, string(mj.Rules))
	if mj.ReadOnly {
		buf.WriteString(`,"readOnly":true`)
	} else {
		buf.WriteString(`,"readOnly":false`)
	}
	if mj.TextOnly {
		buf.WriteString(`,"textOnly":true`)
	} else {
		buf.WriteString(`,"textOnly":false`)
	}
	if mj.ForcedAnon {
		buf.WriteString(`,"forcedAnon":true`)
	} else {
		buf.WriteString(`,"forcedAnon":false`)
	}
	if mj.HashCommands {
		buf.WriteString(`,"hashCommands":true`)
	} else {
		buf.WriteString(`,"hashCommands":false`)
	}
	buf.WriteByte('}')
	return nil
}

func (mj *BoardConfigs) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *BoardConfigs) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	buf.WriteString(`{"id":`)
	fflib.WriteJsonString(buf, string(mj.ID))
	buf.WriteString(`,"eightball":`)
	if mj.Eightball != nil {
		buf.WriteString(`[`)
		for i, v := range mj.Eightball {
			if i != 0 {
				buf.WriteString(`,`)
			}
			fflib.WriteJsonString(buf, string(v))
		}
		buf.WriteString(`]`)
	} else {
		buf.WriteString(`null`)
	}
	if mj.CodeTags {
		buf.WriteString(`,"codeTags":true`)
	} else {
		buf.WriteString(`,"codeTags":false`)
	}
	buf.WriteString(`,"title":`)
	fflib.WriteJsonString(buf, string(mj.Title))
	buf.WriteString(`,"notice":`)
	fflib.WriteJsonString(buf, string(mj.Notice))
	buf.WriteString(`,"rules":`)
	fflib.WriteJsonString(buf, string(mj.Rules))
	if mj.ReadOnly {
		buf.WriteString(`,"readOnly":true`)
	} else {
		buf.WriteString(`,"readOnly":false`)
	}
	if mj.TextOnly {
		buf.WriteString(`,"textOnly":true`)
	} else {
		buf.WriteString(`,"textOnly":false`)
	}
	if mj.ForcedAnon {
		buf.WriteString(`,"forcedAnon":true`)
	} else {
		buf.WriteString(`,"forcedAnon":false`)
	}
	if mj.HashCommands {
		buf.WriteString(`,"hashCommands":true`)
	} else {
		buf.WriteString(`,"hashCommands":false`)
	}
	buf.WriteByte('}')
	return nil
}

func (mj *BoardPublic) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *BoardPublic) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	if mj.CodeTags {
		buf.WriteString(`{"codeTags":true`)
	} else {
		buf.WriteString(`{"codeTags":false`)
	}
	buf.WriteString(`,"title":`)
	fflib.WriteJsonString(buf, string(mj.Title))
	buf.WriteString(`,"notice":`)
	fflib.WriteJsonString(buf, string(mj.Notice))
	buf.WriteString(`,"rules":`)
	fflib.WriteJsonString(buf, string(mj.Rules))
	if mj.ReadOnly {
		buf.WriteString(`,"readOnly":true`)
	} else {
		buf.WriteString(`,"readOnly":false`)
	}
	if mj.TextOnly {
		buf.WriteString(`,"textOnly":true`)
	} else {
		buf.WriteString(`,"textOnly":false`)
	}
	if mj.ForcedAnon {
		buf.WriteString(`,"forcedAnon":true`)
	} else {
		buf.WriteString(`,"forcedAnon":false`)
	}
	if mj.HashCommands {
		buf.WriteString(`,"hashCommands":true`)
	} else {
		buf.WriteString(`,"hashCommands":false`)
	}
	buf.WriteByte('}')
	return nil
}

func (mj *BoardTitle) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *BoardTitle) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	buf.WriteString(`{"id":`)
	fflib.WriteJsonString(buf, string(mj.ID))
	buf.WriteString(`,"title":`)
	fflib.WriteJsonString(buf, string(mj.Title))
	buf.WriteByte('}')
	return nil
}

func (mj *Configs) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *Configs) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	if mj.PruneThreads {
		buf.WriteString(`{"pruneThreads":true`)
	} else {
		buf.WriteString(`{"pruneThreads":false`)
	}
	if mj.PruneBoards {
		buf.WriteString(`,"pruneBoards":true`)
	} else {
		buf.WriteString(`,"pruneBoards":false`)
	}
	if mj.Pyu {
		buf.WriteString(`,"pyu":true`)
	} else {
		buf.WriteString(`,"pyu":false`)
	}
	buf.WriteString(`,"JPEGQuality":`)
	fflib.FormatBits2(buf, uint64(mj.JPEGQuality), 10, false)
	buf.WriteString(`,"maxWidth":`)
	fflib.FormatBits2(buf, uint64(mj.MaxWidth), 10, false)
	buf.WriteString(`,"maxHeight":`)
	fflib.FormatBits2(buf, uint64(mj.MaxHeight), 10, false)
	buf.WriteString(`,"threadExpiry":`)
	fflib.FormatBits2(buf, uint64(mj.ThreadExpiry), 10, false)
	buf.WriteString(`,"boardExpiry":`)
	fflib.FormatBits2(buf, uint64(mj.BoardExpiry), 10, false)
	buf.WriteString(`,"maxSize":`)
	fflib.FormatBits2(buf, uint64(mj.MaxSize), 10, false)
	buf.WriteString(`,"sessionExpiry":`)
	fflib.FormatBits2(buf, uint64(mj.SessionExpiry), 10, false)
	buf.WriteString(`,"rootURL":`)
	fflib.WriteJsonString(buf, string(mj.RootURL))
	buf.WriteString(`,"salt":`)
	fflib.WriteJsonString(buf, string(mj.Salt))
	buf.WriteString(`,"feedbackEmail":`)
	fflib.WriteJsonString(buf, string(mj.FeedbackEmail))
	buf.WriteString(`,"captchaPrivateKey":`)
	fflib.WriteJsonString(buf, string(mj.CaptchaPrivateKey))
	buf.WriteString(`,"FAQ":`)
	fflib.WriteJsonString(buf, string(mj.FAQ))
	if mj.Captcha {
		buf.WriteString(`,"captcha":true`)
	} else {
		buf.WriteString(`,"captcha":false`)
	}
	if mj.Mature {
		buf.WriteString(`,"mature":true`)
	} else {
		buf.WriteString(`,"mature":false`)
	}
	buf.WriteString(`,"defaultLang":`)
	fflib.WriteJsonString(buf, string(mj.DefaultLang))
	buf.WriteString(`,"defaultCSS":`)
	fflib.WriteJsonString(buf, string(mj.DefaultCSS))
	buf.WriteString(`,"captchaPublicKey":`)
	fflib.WriteJsonString(buf, string(mj.CaptchaPublicKey))
	buf.WriteString(`,"imageRootOverride":`)
	fflib.WriteJsonString(buf, string(mj.ImageRootOverride))
	if mj.Links == nil {
		buf.WriteString(`,"links":null`)
	} else {
		buf.WriteString(`,"links":{ `)
		for key, value := range mj.Links {
			fflib.WriteJsonString(buf, key)
			buf.WriteString(`:`)
			fflib.WriteJsonString(buf, string(value))
			buf.WriteByte(',')
		}
		buf.Rewind(1)
		buf.WriteByte('}')
	}
	buf.WriteByte('}')
	return nil
}

func (mj *PostParseConfigs) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *PostParseConfigs) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	if mj.ReadOnly {
		buf.WriteString(`{"readOnly":true`)
	} else {
		buf.WriteString(`{"readOnly":false`)
	}
	if mj.TextOnly {
		buf.WriteString(`,"textOnly":true`)
	} else {
		buf.WriteString(`,"textOnly":false`)
	}
	if mj.ForcedAnon {
		buf.WriteString(`,"forcedAnon":true`)
	} else {
		buf.WriteString(`,"forcedAnon":false`)
	}
	if mj.HashCommands {
		buf.WriteString(`,"hashCommands":true`)
	} else {
		buf.WriteString(`,"hashCommands":false`)
	}
	buf.WriteByte('}')
	return nil
}

func (mj *Public) MarshalJSON() ([]byte, error) {
	var buf fflib.Buffer
	if mj == nil {
		buf.WriteString("null")
		return buf.Bytes(), nil
	}
	err := mj.MarshalJSONBuf(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
func (mj *Public) MarshalJSONBuf(buf fflib.EncodingBuffer) error {
	if mj == nil {
		buf.WriteString("null")
		return nil
	}
	var err error
	var obj []byte
	_ = obj
	_ = err
	if mj.Captcha {
		buf.WriteString(`{"captcha":true`)
	} else {
		buf.WriteString(`{"captcha":false`)
	}
	if mj.Mature {
		buf.WriteString(`,"mature":true`)
	} else {
		buf.WriteString(`,"mature":false`)
	}
	buf.WriteString(`,"defaultLang":`)
	fflib.WriteJsonString(buf, string(mj.DefaultLang))
	buf.WriteString(`,"defaultCSS":`)
	fflib.WriteJsonString(buf, string(mj.DefaultCSS))
	buf.WriteString(`,"captchaPublicKey":`)
	fflib.WriteJsonString(buf, string(mj.CaptchaPublicKey))
	buf.WriteString(`,"imageRootOverride":`)
	fflib.WriteJsonString(buf, string(mj.ImageRootOverride))
	if mj.Links == nil {
		buf.WriteString(`,"links":null`)
	} else {
		buf.WriteString(`,"links":{ `)
		for key, value := range mj.Links {
			fflib.WriteJsonString(buf, key)
			buf.WriteString(`:`)
			fflib.WriteJsonString(buf, string(value))
			buf.WriteByte(',')
		}
		buf.Rewind(1)
		buf.WriteByte('}')
	}
	buf.WriteByte('}')
	return nil
}