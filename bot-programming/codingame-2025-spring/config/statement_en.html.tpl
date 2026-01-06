<!-- LEAGUES level1 level2 level3 level4 level5 -->

<div id="statement_back" class="statement_back" style="display: none"></div>
<div class="statement-body">
  <!-- LEAGUE ALERT -->
  <div
    style="
      color: #7cc576;
      background-color: rgba(124, 197, 118, 0.1);
      padding: 20px;
      margin-right: 15px;
      margin-left: 15px;
      margin-bottom: 10px;
      text-align: left;
    "
  >
    <div style="text-align: center; margin-bottom: 6px">
      <img
        src="//cdn.codingame.com/smash-the-code/statement/league_wood_04.png"
      />
    </div>
    <p style="text-align: center; font-weight: 700; margin-bottom: 6px">
      This is a <b>league based</b> challenge.
    </p>
    <span class="statement-league-alert-content">
      For this challenge, multiple leagues for the same game are available. Once
      you have proven your skills against the first Boss, you will access a
      higher league and extra rules will be available.
      <br /><br />
      <b>NEW:</b> In wooden leagues, your submission will only fight the boss in
      the arena. Complete the objective specified in each league at least 3
      times out 5 to advance to the next league.
    </span>
  </div>

  <!-- GOAL -->
  <div class="statement-section statement-goal">
    <h2>
      <span class="icon icon-goal">&nbsp;</span>
      <span>Goal</span>
    </h2>
    <div class="statement-goal-content">
      <div>
        <!-- BEGIN level5 -->
        Win the water fight by controlling the most territory, or out-soak your
        opponent!
        <!-- END -->

        <!-- BEGIN level1 -->
        In this league, move one of your agents to the coordinates
        <const>(6,1)</const>, and the other to <const>(6,3)</const>.
        <!-- END -->

        <!-- BEGIN level2 -->
        In this league, <strong>shoot the enemy agent with the highest <var>wetness</var> on each turn</strong> using both your agents.
        <!-- END -->

        <!-- BEGIN level3 -->
        <p>
          In this league, you have <strong>1 turn</strong> to get
          both your agents behind the best cover
          then shoot the opposing enemy with the least protection from cover.
        </p>
        <!-- END -->

        <!-- BEGIN level4 -->
        In this league, eliminate all three groups of
        only enemy agents with your <b>splash bomb</b> supply.
        <!-- END -->
      </div>
    </div>
  </div>

  <!-- RULES -->
  <div class="statement-section statement-rules">
    <h2>
      <span class="icon icon-rules">&nbsp;</span>
      <span>Rules</span>
    </h2>

    <div class="statement-rules-content">
      <p>The game is played on a <b>grid</b>.</p>
      <p>Each player controls a team of <b>agents</b>.</p>

      <!-- BEGIN level5 -->
      <div class="full-statement-a">
        <p>
          Each turn, every agent can perform one
          <strong>move action</strong> and/or one
          <strong>combat action</strong>.
        </p>
        <br />

        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🔥 Agents
        </h3>
        <p>
          Agents are the player-controlled units on the field. They possess
          <b>attributes</b> and actions.
        </p>

        <p>
          Each agent has a <var>wetness</var> meter, which goes up when getting
          attacked by enemy agents. Once an agent’s <var>wetness</var> reaches
          100, they are eliminated and removed from play.
        </p>

        <p>
          Agents also have a set <var>soaking_power</var> and
          <var>optimal_range</var>. The power indicates how much wetness they
          normally deal, while the range is used to apply a penalty if the
          target is too far.
        </p>

        <ul>
          <li>
            Up to the <var>optimal_range</var>, <b>shooting</b> deals 100% of
            their <var>soaking_power</var>.
          </li>
          <li>
            Beyond that, and up to twice the <var>optimal_range</var>,
            <b>shooting </b>only deals 50% of their <var>soaking_power</var>.
          </li>
          <li>Beyond that, the shot fails.</li>
        </ul>
        <p>
          <em
            >Note: All distances are calculated with the Manhattan formula.</em
          >
        </p>

        <p>
          Each agent also has a <var>shoot_cooldown</var>, which is the amount
          of turns they must wait after <b>shooting</b> to be able to
          <action>SHOOT</action> again. They can still use other actions in the
          meantime.
        </p>

        <p>
          In addition to shooting, each agent has a set amount of splash
          <var>bombs</var> that they can throw. The amount is determined at the
          start of the game and can differ between agents.
        </p>
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🎬 Actions
        </h3>

        <p>
          On each turn, you must output one command for each agent that you
          control. Each command can contain several actions; at most one
          <b>move action</b> and one <b>combat action</b>.<br /><br />
          You can instruct the actions in any order you want, but the execution
          order will depend on each action’s priority; see the
          <strong>Action order per turn</strong> section for more details.
        </p>

        <p>
          Moving is done with the <action>MOVE x y</action> command. With it,
          the agent will attempt to move to the location x, y. If the target
          location is not orthogonally adjacent to the agent, then they will
          attempt to move towards it using the shortest valid path possible. If
          the action results in a movement on a tile with a cover or another
          agent on it, the movement will be cancelled. If agents collide while
          attempting to <action>MOVE</action>, their movement will be cancelled.
        </p>

        <p></p>

        <p>There are several combat actions available:</p>

        <ul>
          <li>
            <action>SHOOT id</action>: Attempt to shoot agent
            <var>agentId</var>. This will deal <var>wetness</var> according to
            the <var>optimalRange</var> and <var>soakingPower</var> of the
            agent, and is reduced by any damage reduction gained by the target
            agent (see the <action>HUNKER_DOWN</action>action and the
            <strong>Cover</strong> section).
          </li>
          <li>
            <action>THROW x y</action>: Attempt to throw a <b>splash bomb</b> at
            the location <var>x</var>, <var>y</var>. <b>Splash bombs</b> can
            only be thrown at a maximum distance of <const>4</const> tiles away
            from the agent. They deal <const>30</const> <var>wetness</var> to
            agents on the tile it lands on and to all adjacent tiles
            (orthogonally and diagonally). This action <b>ignores</b> damage
            reduction from covers & hunkering.
          </li>
          <li>
            <action>HUNKER_DOWN</action>: Hunker down to gain 25% damage
            reduction against enemy shots this turn. This can be stacked with
            cover bonuses (see the <strong>Cover</strong> section below).
          </li>
        </ul>
        <p></p>

        <p>
          <em
            >See the <strong>Game Protocol </strong>section for more information
            on sending commands to your agents.</em
          >
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level4 -->
      <div class="tutorial-4">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objective 4: Throwing splash bombs
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=147595400748837"
            style="width: 60%; max-width: 300px"
          />
        </div>
        <p>
          Your agents can now run and gun behind <b>cover</b>! In this new
          league, throw <strong>splash bombs</strong> at enemies to deal massive
          wetness damage <strong>regardless of cover</strong>.
        </p>
        <bR />
        <p>
          Agents will sometimes start the game with a number of
          <b>splash bombs</b>. The current amount of <b>splash bombs</b> for any
          given agent is given each turn in the standard input as the
          <var>splashBombs</var> variable.
        </p>
        <br />
        <p>
          Throwing a splash bomb is a <strong>combat action</strong>. Meaning it
          can be used after a <strong>MOVE</strong> action, just like the
          <strong>SHOOT</strong> action.
        </p>

        <br />

        <p>
          An agent using the <action>THROW x y</action> action will attempt to
          throw a splash bomb at the location <var>x</var>, <var>y</var>.
          <b>Splash bombs</b> can only be thrown at a <b>maximum distance</b> of
          <const>4</const> tiles away from the agent. They deal 30
          <var>wetness</var> to the tile it lands on, and 30
          <var>wetness</var> to all adjacent tiles (orthogonally and
          diagonally).
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level3 -->
      <div class="tutorial-3-a">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objective 3: Taking Cover
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554441949158"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Your agents can shoot enemy agents! In this next league, you will have
          to deal with <strong>cover</strong>.
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level3 level5 -->
      <div class="cover-explanation">
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🛡️ Cover
        </h3>
        <p>
          Each tile of the <b>grid</b> is given to your program through the
          standard input. For each column of each row you are given a
          <var>tileType</var>. It can now have one of <b>three</b> possible
          values:
        </p>
        <ul>
          <li><const>0</const> an empty tile.</li>
        </ul>
        <ul>
          <li>
            <const>1</const> a tile containing <strong>low cover.</strong>
          </li>
        </ul>
        <ul>
          <li>
            <const>2</const> a tile containing <strong>high cover.</strong>
          </li>
        </ul>
        <p>
          Tiles with <strong>cover</strong> are impassable, and agents will
          automatically path around them when perform a
          <action>MOVE</action> action.
        </p>
        <br />

        <figure>
          <img
            style="width: 60%; max-width: 80px; margin-bottom: 10px"
            src="https://static.codingame.com/servlet/fileservlet?id=145594581729355"
          />
        </figure>
        <p>
          An agent that benefits from a cover will gain damage reduction against
          enemy shots. Low Covers provide <const>50%</const> protection, and
          High Covers provide <const>75%</const> protection.
        </p>
        <p>
          <em
            >For instance, an agent within optimal range and a soaking power of
            <const>24</const> will only deal <const>6</const> wetness to an
            enemy behind High Cover.</em
          >
        </p>
        <p>
          To benefit from a cover, the agent must be orthogonally adjacent to
          it, and the enemy shot must come from the opposite side of the cover
          tile. The cover is ignored if both agents are adjacent to the cover.
        </p>
        <p>
          In the case where multiple covers can be considered, only the
          <b>highest</b> cover will count.
        </p>
        <br />
        <p><strong>Examples:</strong></p>
        <div class="statement-example-container">
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594221358130"
            />
            <div class="legend">
              <div class="description">
                An agent orthogonally adjacent to the left side of a low cover.
                Shooting this agent from a green tile will result in damage
                reduction.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594162353521"
            />
            <div class="legend">
              <div class="description">
                In the case where a shot can be affected by two covers, only the
                highest one counts.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594244448576"
            />
            <div class="legend">
              <div class="description">
                An agent that is not orthogonally adjacent to any cover, thus
                not benefitting from their damage reduction.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594209782393"
            />
            <div class="legend">
              <div class="description">
                The orange agent benefits from a low cover while the purple
                agent does not benefit from any cover.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594180818284"
            />
            <div class="legend">
              <div class="description">
                Neither of these agents benefit from the cover from each other
                since they are both adjacent to it.
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- END -->

      <!-- BEGIN level5 -->
      <div class="full-statement-b">
        <p>
          <em
            >Note: Hunkering down stacks with cover, which means the total
            protection gained from both taking cover and hunkering down is 75%
            or 100%, for low and high cover respectively.</em
          >
        </p>
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          📈 Points
        </h3>
        <p>
          You gain points by taking control of tiles when you control a larger
          area than your opponent.
        </p>

        <p>
          Any tile that is closer to one of your agents than to an enemy agent
          is considered to be under your control. However, if an agent has
          <var>wetness</var> greater or equal to <const>50</const>, the distance
          to that agent will be <strong>doubled</strong> during this comparison.
        </p>

        <p>
          Each turn, if you control <strong>more</strong> tiles than your
          opponent, you score as many points as extra tiles that you control
          compared to your opponent.
        </p>

        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🎬 Action order for one turn
        </h3>
        <p>
          Game turns are synchronous, for both players and agents (meaning all
          agents perform their actions at the same time). However, some actions
          have priority over others:
        </p>

        <ul>
          <li><action>MOVE</action> actions go first,</li>
          <li>Then <action>HUNKER_DOWN</action> actions,</li>
          <li>Then <action>SHOOT</action> and <action>THROW</action>,</li>
          <li>And finally, the removal of any soaked agent.</li>
        </ul>
        <br />
      </div>
      <!-- END -->

      <!-- BEGIN level3 -->
      <div class="tutorial-3-b">
        <h2>Run &amp; Gun</h2>
        <p>
          From this league onwards, your agents may perform <b>both</b> a
          <action>MOVE</action> and <action>SHOOT</action> action on the same
          turn. Separate both actions by a semicolon and the assigned agent will
          first perform the <action>MOVE</action> then immediately attempt to
          <action>SHOOT</action> from the new position.
        </p>
        <p>
          Example:
          <action><code>1; MOVE 6 3; SHOOT 4</code></action>
        </p>
        <br />

        In this league, you will have agents on either side of the screen. They
        will both be confronted by two enemy agents in range. They will also be
        <const>1</const> <action>MOVE</action> away from tiles with cover. To
        beat this league, you must move both agents behind the highest available
        cover and have them shoot the enemy within range behind the lowest
        cover.
      </div>
      <!-- END -->

      <!-- BEGIN level2 -->
      <div class="tutorial-2">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objective 2: the <action>SHOOT</action> action
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554402703235"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Your agents can move! In this next league, <b>enemy agents</b> have
          entered the field!
          <br />
          <br />Thankfully, your agents are also capable of performing the
          <action>SHOOT</action> action.
          <br />
          <br />In this game, agents can shoot each other with
          <strong>water guns</strong>. Shooting an agent will increase its
          <var>wetness</var>.If an agent's wetness reaches <const>100</const> or
          more, they are removed from the game.
        </p>
        <p>
          The amount of <var>wetness</var> added to an agent when shot is equal
          to the <var>soakingPower</var> of the shooter. This can be refered to
          as <b>damage</b>.
        </p>
        <p>
          However, that amount will be <strong>halved</strong> if the
          <b>manhattan distance</b> separating the two agents is greater than
          the <var>optimalRange </var>of the shooter. The shot will
          <b>fail</b> if the distance is greater than twice the
          <var>optimalRange</var>, in which case no damage is dealt.
        </p>
        <p></p>
        <p>
          Enemy agents will be present in the list of agents in the standard
          input. You may identify them with the <var>player</var> variable. You are also given their <var>agentId</var> and <var>wetness</var>. The agents with a value <var>player</var> that equals <var>myId</var> are yours.
        </p>
        <p></p>
        <p>
          The <action>SHOOT id</action> action will tell an agent to shoot the
          agent with the given id. Each agent can perform one
          <action>SHOOT</action> action per turn.
        </p>
        <br />
      </div>
      <!-- END -->

      <!-- BEGIN level1 -->
      <div class="tutorial-1">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objective 1: the <action>MOVE</action> action
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554460332080"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Each one of your agents occupy a tile on the grid. They cannot occupy
          the same tile. Each agent has a unique
          <var>agentId</var>.
        </p>
        <p>
          Each agent can perform one <action>MOVE</action> action per turn. By
          printing a <action> MOVE x y</action> action to the standard output,
          you can tell an agent to move one tile towards the given coordinates.
        </p>

        <p>
          To assign an action to an agent, print to the standard output its
          <var>agentId</var> followed by the desired action, the two separated
          by a <b>semicolon</b>. <br /><br />
          For example, the following line:<br />
          <action>1; MOVE 12 3</action><br />
          will assign the <action>MOVE 12 3</action> to the agent with
          <var>agentId</var> = 1.
        </p>

        <br />

        <p>
          You must send exactly <const>1</const> line per agent on your team.
        </p>

        <p>
          <em>
            Grid and agent data are provided to your program through the
            standard input. Further details in the Game Protocol section.
          </em>
        </p>
      </div>
      <!-- END -->

      <!-- Victory conditions -->
      <div class="statement-victory-conditions">
        <div class="icon victory"></div>
        <div class="blk">
          <div class="title">Victory Conditions</div>

          <div class="text">
            <!-- BEGIN level1 -->
            In this league you have <const>two</const> agents on a small
            grid.<br />
            Your objective is to move one of your agents to the coordinates
            <const>(6,1)</const>, and the other to <const>(6,3)</const>.
            <!-- END -->
            <!-- BEGIN level2 -->
            In this league you have two agents on a small grid. Your objective
            is to
            <strong
              >shoot the enemy agent with the highest wetness on each turn </strong
            >using both your agents.
            <!-- END -->

            <!-- BEGIN level3 -->
            <p>
              In this league, you will have exactly <strong>1 turn </strong>to
              get both your agents behind the best of two adjacent tiles behind
              cover then shoot the opposing enemy with the least protection from
              cover (of the two closest enemies).
            </p>
            <!-- END -->

            <!-- BEGIN level4 -->
            In this league, there are four groups of barricaded agents, one of
            which includes one of your own agents. You must eliminate all three
            groups of only enemy agents with your limited splash bomb supply.
            <b>Shooting is disabled</b>.
            <!-- END -->
            <!-- BEGIN level5 -->
            <p>
              The winner is the player who fulfills one of the following
              conditions:
            </p>

            <ul>
              <li>
                Reach <const>600</const> <b>more</b> points than their opponent
              </li>
              <li>Eliminate all opposing agents</li>
              <li>Have the most points after <const>100</const> turns</li>
            </ul>
            <!-- END -->
          </div>
        </div>
      </div>
      <!-- Lose conditions -->
      <div class="statement-lose-conditions">
        <div class="icon lose"></div>
        <div class="blk">
          <div class="title">Defeat Conditions</div>
          <div class="text">
            <!-- BEGIN level1 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li><const>20 turns</const> have passed.</li>
              <li>
                Your program does not provide a command in the alloted time or
                one of the commands is invalid.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level2 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>
                One or more of your agents does not shoot the wettest foe.
              </li>
              <li>
                Your program does not provide a command in the alloted time or
                one of the commands is invalid.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level3 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>
                Either of your agents moves to the incorrect location or fails
                to shoot the correct foe.
              </li>
              <li>
                Your program does not provide a command in the alloted time or
                one of the commands is invalid.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level4 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>You hit any of your own agents.</li>
              <li><const>40 turns</const> have passed.</li>
              <li>
                Your program does not provide a command in the alloted time or
                one of the commands is invalid.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level5 -->
            Your program does not provide a command in the alloted time or one
            of the commands is invalid.
            <!-- END -->
          </div>
        </div>
      </div>

      <br />
      <h3
        style="
          font-size: 14px;
          font-weight: 700;
          padding-top: 5px;
          padding-bottom: 15px;
        "
      >
        🐞 Debugging tips
      </h3>
      <ul>
        <li>
          Hover over the grid to see extra information on the tile under your
          mouse.
        </li>
        <li>
          Assign the special <action>MESSAGE text</action> action to an agent
          and that text will appear above your agent.
        </li>
        <li>
          Press the gear icon on the viewer to access extra display options.
        </li>
        <li>
          Use the keyboard to control the action: space to play/pause, arrows to
          step 1 frame at a time.
        </li>
      </ul>
    </div>
  </div>

  <!-- PROTOCOL -->

  <!-- BEGIN level1 level2 level3 level4 -->
  <details class="statement-section statement-protocol">
    <!-- END -->
    <!-- BEGIN level5 -->
    <details open class="statement-section statement-protocol">
      <!-- END -->
      <summary
        open
        style="cursor: pointer; margin-bottom: 10px; display: inline-block"
      >
        <span style="display: inline-block; margin-bottom: 10px"
          >Click to expand</span
        >
        <h2 style="margin-bottom: 0">
          <span class="icon icon-protocol">&nbsp;</span>
          <span>Game Protocol</span>
        </h2>
      </summary>

      <!-- Protocol block -->
      <div class="blk">
        <div class="title">Initialization Input</div>
        <div class="text">
          <span class="statement-lineno">First line:</span> one integer
          <var>myId</var>, for your player identification.
          <p>
            <span class="statement-lineno">Second line:</span> one integer
            <var>agentDataCount</var> for the number of agents on the grid.
          </p>
          <p>
            <span class="statement-lineno">Next </span><var>agentDataCount</var>
            <span class="statement-lineno">lines:</span> The following
            <const>6</const> inputs for each agent:
          </p>
          <ul>
            <li><var>agentId</var>: unique id of this agent</li>
            <li><var>player</var>: id of the player owning this agent</li>
            <li>
              <var>shootCooldown</var>: min number or turns between two shots
              for this agent
            </li>
            <li>
              <var>optimalRange</var>: the optimal shooting range of this agent
            </li>
            <li>
              <var>soakingPower</var>: the maximum wetness damage output of this
              agent
            </li>
            <li>
              <var>splashBombs</var>: the starting amount of splash bombs
              available to this agent
            </li>
          </ul>
          <p>
            <span class="statement-lineno">Next line:</span> two integers
            <var>width</var> and
            <var>height</var>
            for the size of the grid.
          </p>
          <p>
            <span class="statement-lineno">The next </span><var>width</var
            ><span class="statement-lineno"> * </span><var>height</var>
            <span class="statement-lineno">lines:</span> The following 3 inputs
            for each tile on the grid:
          </p>
          <ul>
            <li><var>x</var>: X coordinate (0 is leftmost)</li>
            <li><var>y</var>: Y coordinate (0 is uppermost)</li>
            <li>
              <var>tile_type</var>:
              <ul>
                <li><const>0</const> for an empty tile</li>
                <li><const>1</const> for a low cover</li>
                <li><const>2</const> for a high cover</li>
              </ul>
            </li>
          </ul>
        </div>
      </div>
      <div class="blk">
        <div class="title">Input for one game turn</div>
        <div class="text">
          <p>
            <span class="statement-lineno">First line:</span> one integer
            <var>agentCount</var> for the number of remaining agents on the
            grid.
          </p>
          <p>
            <span class="statement-lineno">Next </span><var>agentCount</var>
            <span class="statement-lineno">lines:</span> The following
            <const>6</const> inputs for each agent:
          </p>
          <ul>
            <li><var>agentId</var>: unique id of this agent</li>
            <li><var>x</var>: X coordinate (0 is leftmost)</li>
            <li><var>y</var>: Y coordinate (0 is uppermost)</li>
            <li>
              <var>cooldown</var>: number of turns left until this agent can
              shoot again
            </li>
            <li>
              <var>splashBombs</var>: current amount of splash bombs available
              to this agent
            </li>
            <li><var>wetness</var>: current wetness of the agent</li>
          </ul>
          <p>
            <span class="statement-lineno">Next line:</span> one integer
            <var>myAgentCount</var> for the number of agents controlled by the
            player.
          </p>
        </div>
      </div>

      <!-- Protocol block -->
      <div class="blk">
        <div class="title">Output</div>
        <div class="text">
          <p>
            A single line per agent, preceded by its <var>agentId</var> and
            followed by its action(s):
          </p>
          <em>Up to one move action:</em>
          <ul>
            <li>
              <action>MOVE x y</action>: Attempt to move towards the location
              <var>x</var>, <var>y</var>.
            </li>
          </ul>
          <em>Up to one combat action:</em>
          <ul>
            <li>
              <action>SHOOT id</action>: Attempt to shoot agent
              <var>agentId</var>.
            </li>
            <li>
              <action>THROW</action>: Attempt to throw a splash bomb at the
              location <var>x</var>, <var>y</var>.
            </li>
            <li>
              <action>HUNKER_DOWN</action>: Hunker down to gain 25% damage
              reduction against enemy attacks this turn.
            </li>
          </ul>
          <em>Up to one message action:</em>
          <ul>
            <li>
              <action>MESSAGE text</action>: Display <var>text</var> in the
              viewer. Useful for debugging.
            </li>
          </ul>
          <p>
            Instructions are separated by semicolons. For example, consider
            the following line:
          </p>
          <p>
            <action><code>3;MOVE 12 3;SHOOT 5</code></action>
          </p>
          <p>
            This instructs <strong>agent 3</strong> to
            <strong>move towards the coordinates (12, 3)</strong> and to
            <strong>shoot agent 5</strong>.
          </p>
          <p>
            <em
              ><strong>Note:</strong> The <var>agentId</var> at the start can be
              omitted. In that case, the actions are assigned to the agents in
              ascending order of <var>agentId</var>.</em
            >
          </p>
        </div>
      </div>

      <div class="blk">
        <div class="title">Constraints</div>
        <div class="text">
          Response time per turn ≤ <const>50</const>ms <br />Response time for
          the first turn ≤ <const>1000</const>ms
          <!-- BEGIN level5 -->
          <br />
          <const>12</const> &le; <var>width</var> &le; <const>20</const>
          <br />
          <const>6</const> &le; <var>height</var> &le; <const>10</const>
          <br />
          <const>6</const> &le; <var>agentDataCount</var> &le; <const>10</const>
          <!-- END -->
        </div>
      </div>

      <!-- BEGIN level1 level2 level3 level4 -->
    </details>
    <!-- END -->
    <!-- BEGIN level5 -->
  </details>
  <!-- END -->

  <!-- SHOW_SAVE_PDF_BUTTON -->
</div>
