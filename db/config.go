// Configuration loading, reloading and setting

package db

import (
	"database/sql"
	"encoding/json"
	"log"
	"time"

	"github.com/bakape/meguca/config"
	"github.com/bakape/meguca/templates"
	"github.com/bakape/meguca/util"
	"github.com/lib/pq"
)

// DatabaseBoardConfigs contains extra fields not exposed on database reads
type DatabaseBoardConfigs struct {
	config.BoardConfigs
	Created time.Time
}

// Load configs from the database and update on each change
func loadConfigs() error {
	conf, err := GetConfigs()
	if err != nil {
		return err
	}
	config.Set(conf)

	return listen("config_updates", updateConfigs)
}

// GetConfigs retrieves global configurations. Only used in tests.
func GetConfigs() (c config.Configs, err error) {
	var enc string
	err = db.QueryRow(`SELECT val FROM main WHERE id = 'config'`).Scan(&enc)
	if err != nil {
		return
	}
	c, err = decodeConfigs(enc)
	return
}

// Assigns a function to listen to Postgres notifications
func listen(event string, fn func(msg string) error) error {
	if IsTest {
		return nil
	}

	l := pq.NewListener(
		ConnArgs,
		time.Second,
		time.Second*10,
		func(_ pq.ListenerEventType, _ error) {},
	)
	if err := l.Listen(event); err != nil {
		return err
	}

	go func() {
		for msg := range l.Notify {
			if err := fn(msg.Extra); err != nil {
				log.Println(err)
			}
		}
	}()

	return nil
}

func decodeConfigs(data string) (c config.Configs, err error) {
	err = json.Unmarshal([]byte(data), &c)
	return
}

func loadBoardConfigs() error {
	r, err := prepared["get_board_configs"].Query()
	if err != nil {
		return err
	}
	defer r.Close()

	for r.Next() {
		c, err := scanBoardConfigs(r)
		if err != nil {
			return err
		}
		if _, err := config.SetBoardConfigs(c); err != nil {
			return err
		}
	}
	if err := r.Err(); err != nil {
		return err
	}

	return listen("board_updated", updateBoardConfigs)
}

func scanBoardConfigs(r rowScanner) (c config.BoardConfigs, err error) {
	var eightball pq.StringArray
	err = r.Scan(
		&c.ReadOnly, &c.TextOnly, &c.ForcedAnon, &c.HashCommands, &c.CodeTags,
		&c.ID, &c.Title, &c.Notice, &c.Rules, &eightball,
	)
	c.Eightball = []string(eightball)
	return
}

// WriteBoard writes a board complete with configurations to the database
func WriteBoard(tx *sql.Tx, c DatabaseBoardConfigs) error {
	_, err := getStatement(tx, "write_board").Exec(
		c.ReadOnly, c.TextOnly, c.ForcedAnon, c.HashCommands, c.CodeTags, c.ID,
		c.Created, c.Title, c.Notice, c.Rules, pq.StringArray(c.Eightball),
	)
	return err
}

// UpdateBoard updates board configurations
func UpdateBoard(c config.BoardConfigs) error {
	_, err := prepared["update_board"].Exec(
		c.ID, c.ReadOnly, c.TextOnly, c.ForcedAnon, c.HashCommands, c.CodeTags,
		c.Title, c.Notice, c.Rules, pq.StringArray(c.Eightball),
	)
	return err
}

// WriteStaff writes staff positions of a specific board. Old rows are
// overwritten. tx must not be nil.
func WriteStaff(tx *sql.Tx, board string, staff map[string][]string) error {
	// Remove previous staff entries
	_, err := prepared["clear_staff"].Exec(board)
	if err != nil {
		return err
	}

	// Write new ones
	q := tx.Stmt(prepared["write_staff"])
	for pos, accounts := range staff {
		for _, a := range accounts {
			_, err = q.Exec(board, a, pos)
			if err != nil {
				return err
			}
		}
	}

	return nil
}

func updateConfigs(data string) error {
	conf, err := decodeConfigs(data)
	if err != nil {
		return util.WrapError("reloading configuration", err)
	}
	config.Set(conf)

	return recompileTemplates()
}

func updateBoardConfigs(board string) error {
	conf, err := GetBoardConfigs(board)
	switch err {
	case nil:
	case sql.ErrNoRows:
		config.RemoveBoard(board)
		return recompileTemplates()
	default:
		return err
	}

	changed, err := config.SetBoardConfigs(conf)
	switch {
	case err != nil:
		return util.WrapError("reloading board configuration", err)
	case changed:
		return recompileTemplates()
	default:
		return nil
	}
}

// GetBoardConfigs retrives the configurations of a specific board. Only used in
// tests.
func GetBoardConfigs(board string) (config.BoardConfigs, error) {
	r := db.QueryRow(`
		SELECT readOnly, textOnly, forcedAnon, hashCommands, codeTags, id,
			title, notice, rules, eightball
		FROM boards
		WHERE id = $1`,
		board,
	)
	return scanBoardConfigs(r)
}

func recompileTemplates() error {
	if IsTest {
		return nil
	}
	if err := templates.Compile(); err != nil {
		return util.WrapError("recompiling templates", err)
	}
	return nil
}

// WriteConfigs writes new global configurations to the database
func WriteConfigs(c config.Configs) error {
	data, err := json.Marshal(c)
	if err != nil {
		return err
	}
	_, err = prepared["write_configs"].Exec(string(data))
	return err
}
