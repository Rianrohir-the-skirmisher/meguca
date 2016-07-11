package websockets

import (
	"github.com/bakape/meguca/config"
	"github.com/bakape/meguca/db"
	"github.com/bakape/meguca/types"
	r "github.com/dancannon/gorethink"
	. "gopkg.in/check.v1"
)

func (*DB) TestCreateThreadOnInvalidBoard(c *C) {
	req := types.ThreadCreationRequest{
		Board: "all",
	}
	c.Assert(insertThread(marshalJSON(req, c), nil), Equals, errInvalidBoard)
}

func (*DB) TestCreateThreadOnReadOnlyBoard(c *C) {
	q := r.Table("boards").Insert(config.BoardConfigs{
		ID: "a",
		PostParseConfigs: config.PostParseConfigs{
			ReadOnly: true,
		},
	})
	c.Assert(db.Write(q), IsNil)

	req := types.ThreadCreationRequest{
		Board: "a",
	}
	c.Assert(insertThread(marshalJSON(req, c), nil), Equals, errReadOnly)
}

func (*DB) TestThreadCreation(c *C) {
	mains := []map[string]interface{}{
		{
			"id":      "info",
			"postCtr": 5,
		},
		{
			"id": "boardCtrs",
		},
	}
	c.Assert(db.Write(r.Table("main").Insert(mains)), IsNil)
	conf := config.BoardConfigs{
		ID: "a",
	}
	c.Assert(db.Write(r.Table("boards").Insert(conf)), IsNil)

	sv := newWSServer(c)
	defer sv.Close()
	cl, wcl := sv.NewClient()
	sendImage := make(chan types.Image)
	cl.AllocateImage = sendImage
	cl.ident.IP = "::1"
	img := types.Image{
		ImageCommon: types.ImageCommon{
			SHA1: "sha1",
		},
	}
	go func() {
		sendImage <- img
	}()

	req := types.ThreadCreationRequest{
		PostCredentials: types.PostCredentials{
			Name:     "name",
			Password: "123",
		},
		Subject: "subject",
		Board:   "a",
		Body:    "body",
	}
	std := types.DatabaseThread{
		ID:      6,
		Subject: "subject",
		Board:   "a",
		Posts: map[int64]types.Post{
			6: types.Post{
				ID:       6,
				OP:       6,
				Board:    "a",
				IP:       "::1",
				Password: "123",
				Body:     "body",
				Name:     "name",
				Image:    &img,
			},
		},
		Log: [][]byte{},
	}

	c.Assert(insertThread(marshalJSON(req, c), cl), IsNil)
	assertMessage(wcl, []byte("01true"), c)

	var thread types.DatabaseThread
	c.Assert(db.One(r.Table("threads").Get(6), &thread), IsNil)

	// Normalize timestamps
	then := thread.BumpTime
	std.BumpTime = then
	std.ReplyTime = then
	post := std.Posts[6]
	post.Time = then
	std.Posts[6] = post

	c.Assert(thread, DeepEquals, std)
}
