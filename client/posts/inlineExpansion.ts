import { posts } from "../state"
import { on, fetchJSON } from "../util"
import options from "../options"
import { Post } from "./model"
import { PostData } from "../common"
import PostView from "./view"

// Expand or contract linked posts inline
async function onClick(e: MouseEvent) {
	// Don't trigger, when user is trying to open in a new tab or inline
	// expansion is disabled
	if (e.which !== 1 || e.ctrlKey || !options.postInlineExpand) {
		return
	}

	e.preventDefault()

	const el = e.target as Element,
		parent = el.parentElement,
		id = parseInt(el.getAttribute("data-id"))

	if (parent.classList.contains("expanded")) {
		return contractPost(id, parent)
	}

	const model = posts.get(id)
	let found = false
	if (model) {
		// Can not create cyclic DOM trees
		if (model.view.el.contains(parent)) {
			return
		}

		found = true
		parent.classList.add("expanded")
		parent.append(model.view.el)
	} else {
		// Fetch external post from server
		const [data] = await fetchJSON<PostData>(`/json/post/${id}`)
		if (data) {
			const model = new Post(data),
				view = new PostView(model, null)
			found = true
			parent.classList.add("expanded")
			parent.append(view.el)
		}
	}

	if (found) {
		toggleLinkReferences(parent, id, true)
	}
}

// contract and already expanded post and return it to its former position
function contractPost(id: number, parent: HTMLElement) {
	parent.classList.remove("expanded")

	const model = posts.get(id)
	// Fetched from server and not originally part of the thread
	if (!model) {
		return document.getElementById(`p${id}`).remove()
	}


	// Find the ID of the post preceding this one. Make sure the target post is
	// not expanded inline itself.
	const ids = Object.keys(posts.models).sort()
	let i = ids.indexOf(id.toString())
	while (true) {
		const previous = posts.get(parseInt(ids[i - 1]))
		if (!previous) {
			document.getElementById("thread-container").prepend(model.view.el)
			break
		}
		if (previous.view.el.matches("#thread-container > article")) {
			toggleLinkReferences(parent, id, false)
			previous.view.el.before(model.view.el)
			break
		}
		i--
	}
}

// Highlight or unhighlight links referencing the parent post in the child post
function toggleLinkReferences(parent: Element, childID: number, on: boolean) {
	const p = parent.closest("article"),
		ch = document.getElementById(`p${childID}`),
		pID = p.closest("article").id.slice(1)
	for (let el of p.querySelectorAll(".post-link")) {
		// Check if not from a post inlined in the child
		if (el.closest("article") === ch && el.getAttribute("data-id") == pID) {
			el.classList.toggle("referenced", on)
		}
	}
}

export default () =>
	on(document.getElementById("threads"), "click", onClick, {
		selector: ".post-link",
	})

